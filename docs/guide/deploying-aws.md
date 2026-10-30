# Deploying to AWS

`deploy/terraform/` is a reference deployment of the API to AWS
(workspace ADR 0025), the same in both templates. It is a starting point
you own: read it, size it, change it. [The engine's deploying
guide](../../../rok-data-sqlc/docs/guide/deploying.md) covers what any
deployment needs (configuration, the pool, health checks, migrations);
this page is the AWS one.

- [What it makes](#what-it-makes)
- [First deploy](#first-deploy)
- [Each release](#each-release)
- [Testing it without AWS](#testing-it-without-aws)
- [Costs and sizing](#costs-and-sizing)

## What it makes

| Module | |
|---|---|
| `network` | a VPC over two zones; public subnets (the load balancer, one NAT gateway), private ones (the tasks, the database); a security group each for the load balancer, the tasks and the database |
| `database` | RDS PostgreSQL 17, encrypted, backed up 7 days; its password generated and handed on only inside `DATABASE_URL` |
| `storage` | the uploads bucket, private and encrypted, with CORS for direct uploads from `cors_allowed_origins` |
| `mail` | SES: the sending domain's identity, DKIM, and SMTP credentials for `SMTP_URL` |
| `secrets` | Secrets Manager: `JWT_SECRET` and `TOTP_ENCRYPTION_KEY` generated, `DATABASE_URL` and `SMTP_URL` from the modules above |
| `service` | ECR; an ECS Fargate cluster with a `web` service (`APP_ROLE=web`, behind an application load balancer checking `/health/ready`) and a `worker` service (`APP_ROLE=worker`); a `migrate` task definition; CloudWatch logs; the roles (the tasks may use the bucket and nothing else) |

`modules/app` wires them together. `envs/staging` and `envs/prod` call it
with their sizes; prod's database is multi-AZ and protected from
`terraform destroy`. Both keep their state in S3 (`use_lockfile`, no
DynamoDB table needed). The tasks stop the way the API drains: ECS gives
them 40 seconds (`SHUTDOWN_DELAY_SECS` 5 plus `SHUTDOWN_GRACE_SECS` 25,
and a margin), and the load balancer deregisters a stopping task after
30.

## First deploy

1. **The state bucket**, once per account: in `deploy/terraform/bootstrap`,
   `terraform init && terraform apply -var bucket=<name>`. Put the bucket's
   name in `envs/staging/main.tf`'s (and prod's) `backend` block.
2. **The settings**: `envs/staging/terraform.tfvars` (ignored by git):

   ```hcl
   image_tag            = "2026-10-30.1"
   app_url              = "https://app.example.com"
   mail_domain          = "example.com"
   mail_from            = "Example <no-reply@example.com>"
   cors_allowed_origins = ["https://app.example.com"]
   certificate_arn      = "arn:aws:acm:us-east-1:123456789012:certificate/…"
   ```

3. **The images**, built and pushed before the first apply, so the
   services have something to run: see [Each release](#each-release),
   with the repository URL from `terraform apply -target
   module.app.module.service.aws_ecr_repository.this`.
4. **`terraform init && terraform apply`** in `envs/staging`. Commit the
   `.terraform.lock.hcl` the first `init` writes.
5. **Mail**: publish the `mail_dns_records` it prints (a verification TXT
   and three DKIM CNAMEs), and ask AWS to take the account out of the SES
   sandbox before mailing anyone but verified addresses.
6. **The migrations**, before the services take traffic: run the
   `migrate_command` it prints (below).

## Each release

```console
$ REPO=$(terraform output -raw repository_url)   # in envs/staging
$ TAG=2026-10-30.2
$ docker build -t $REPO:$TAG .
$ docker build --target migrate -t $REPO:$TAG-migrate .
$ aws ecr get-login-password | docker login --username AWS --password-stdin ${REPO%%/*}
$ docker push $REPO:$TAG && docker push $REPO:$TAG-migrate
$ terraform apply -var image_tag=$TAG -target module.app.module.service.aws_ecs_task_definition.this
$ eval "$(terraform output -raw migrate_command)"   # wait for exit code 0
$ terraform apply -var image_tag=$TAG
```

The migrations run before the new version starts, so write each one to
work with the version still running (add a column, deploy, then use it;
never rename in one step). The repository's tags are immutable: a tag is
one build. ECS rolls the services, and rolls back a deployment whose tasks
fail their health checks.

## Testing it without AWS

`envs/local` applies the parts [Floci](https://floci.io) emulates — the
bucket, the secrets, mail (the SES identity and the SMTP user), the
database (a real PostgreSQL container) — and `terraform test` checks them:
the bucket's name, the four secrets (and that the generated ones read back
the right size), the database's address, the DNS records. Then it destroys
them.

```console
$ docker compose up -d floci                       # the project's root
$ cd deploy/terraform/envs/local
$ terraform init && terraform test
```

CI runs it, with `terraform fmt -check`, `validate` on every environment
and `tflint`. What it does not prove: the network, the load balancer and
ECS are only validated, and an emulator is not AWS. The first apply to a
real staging account is the last check, every time the Terraform changes.

## Costs and sizing

The fixed costs of an idle staging are the NAT gateway, the load balancer
and the smallest RDS instance; Fargate is per task-hour. One NAT gateway
serves both zones: a zone's outage cuts the tasks in the other zone off
the internet (not off the database); one gateway per zone removes that, at
twice the cost. Size `service.cpu` and `memory` from the
`http_request_duration_seconds` p95 and the pool gauges under load, the
`worker_count` from `jobs_queued{state="due"}`, and the database from its
CPU and connections (`DB_MAX_CONNECTIONS` × tasks stays under its limit).
