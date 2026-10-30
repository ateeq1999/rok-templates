# The templates' guide

`rok sqlc init --template axum` (or `--template actix`) makes an API that
is ready for production: accounts, uploads, background jobs, logs, traces,
metrics, a security baseline, a Docker image, a local stack, CI and an AWS
deployment. This guide is about what the templates ship and how you grow
them; how rok-data-sqlc generates the API itself is
[the engine's guide](../../../rok-data-sqlc/docs/guide/index.md).

Everything here works the same in both templates: the parts with no
framework in them are the same file, byte for byte, in both
(`tests/template_drift.rs` checks it). Each **Capability** is compiled in
and inert until its environment variable turns it on (workspace ADR 0023).

## Pages

What the API does:

- [Accounts and sign-in](auth.md): sign-up, sign-in, the password and
  email flows, two-factor sign-in, API keys, sessions, mail.
- [Files and storage](files.md): uploads, direct uploads, profile photos,
  the storage port, S3.
- [Jobs and schedules](jobs.md): background work on PostgreSQL, scheduled
  jobs, `APP_ROLE`, failed jobs, testing jobs.

Running it in production:

- [Observability](observability.md): logs, the request span, trace export
  (OpenTelemetry), metrics (Prometheus), the local Grafana.
- [Security](security.md): the headers, CORS, rate limits, panics, rotating
  the secrets, the dependency policy.
- [Deploying to AWS](deploying-aws.md): the Terraform, a release, and what
  the Floci test proves and what it does not.

## Recipes

Integrations that pick a vendor or an opt-in dependency are not in the
templates; these pages add them to your project:

- [Errors to Sentry](recipes/sentry.md)
- [Metrics over OTLP](recipes/otel-metrics.md) instead of Prometheus
- [A shared rate limit on Redis](recipes/redis-rate-limit.md) for many
  replicas
- [Signed outbound webhooks](recipes/webhooks.md), delivered as jobs
- [Sign-in with OIDC](recipes/oidc.md) (an identity provider)
