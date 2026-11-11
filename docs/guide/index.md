# The templates' guide

`rok init --template axum` (or `--template actix`) makes an API that is
ready for production: accounts, uploads, background jobs, logs, traces,
metrics, a security baseline, a Docker image, a local stack, CI and an AWS
deployment. `rok init --template monorepo` puts that axum API beside a
web, a desktop and a mobile app, with the clients they call it through.
`rok init --template grpc` serves the axum template's schema over gRPC
instead: tonic services and `.proto` files generated from it, health and
reflection, and the axum template's configuration, data layer, jobs, mail,
storage, telemetry and security, file for file. This guide is about what the templates ship and how you grow them; how
rok-data-sqlc generates the API itself is
[the engine's guide](../../../rok-data-sqlc/docs/guide/index.md).

Everything about the API works the same in every template: the parts with
no framework in them are the same file, byte for byte, in the actix and
axum templates, and the monorepo's `apps/api` is the axum template file
for file (`tests/template_drift.rs` checks both). Each **Capability** is
compiled in and inert until its environment variable turns it on
(workspace ADR 0023).

## Pages

The templates:

- [The monorepo template](monorepo.md): the api, web, desktop and mobile
  Apps, the one contract, the TypeScript, Dart and Rust clients, the
  `rok gen` loop, and what is hidden and why.
- [The gRPC template](../../templates/grpc-api/README.md): the server,
  health and reflection, who is calling, the status mapping, your own
  RPCs, and what it shares with the axum template.

What the API does:

- [Accounts and sign-in](auth.md): sign-up, sign-in, the password and
  email flows, two-factor sign-in, API keys, sessions, mail.
- [Guards](guards.md): authorization rules written once and required by
  type (`Allowed<Finance>`), with a recipe for roles at a resource.
- [Live updates](live.md): Topics, a notice published on a feature's
  transaction and pushed to the subscribed clients, so a screen refetches
  on news instead of polling.
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
- [Push notifications](recipes/push-notifications.md) with Firebase Cloud
  Messaging, for the monorepo's mobile App
