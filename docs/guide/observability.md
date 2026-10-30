# Observability

What the API tells you about itself: log lines, a span for every request
and job, trace export over OpenTelemetry, and metrics in Prometheus's
format. Each is in the template and inert until configured, so a new
project logs to the terminal and exports nothing.

- [Logs](#logs)
- [The request span](#the-request-span)
- [Traces](#traces)
- [Metrics](#metrics)
- [Locally: Jaeger, Prometheus, Grafana](#locally-jaeger-prometheus-grafana)
- [Your own spans and metrics](#your-own-spans-and-metrics)

## Logs

`src/shared/telemetry.rs` starts tracing before anything else, from
`RUST_LOG` (the filter: `info`, `debug,sqlx=warn`) and `LOG_FORMAT`:
`pretty`, one readable line per event (the default), or `json`, one object
per line, for a log collector (the `Dockerfile` sets it). The server logs
its settings at start-up with every secret left out (`JWT_SECRET=<48
bytes>`, a URL's password `<redacted>`), so a wrong setting shows in the
first lines of a deploy.

## The request span

Every request runs in a span carrying its method, its route (the matched
pattern, `/api/v1/users/{id}`, so a backend groups by it), its request id
(answered as `x-request-id`, so a caller can quote it), and, once
`shared/auth.rs` knows it, the caller's `user_id`. Every event logged while
serving the request carries them. A job runs in a span of its own (`job`:
kind, id, attempt), continuing the trace of the request that enqueued it.

## Traces

Set `OTEL_EXPORTER_OTLP_ENDPOINT` (such as `http://otel-collector:4318`)
and every span is exported over OTLP/HTTP; unset, nothing is exported or
connected to. A request carrying a W3C `traceparent` continues the
caller's trace. The exporter reads the standard variables itself:

| Variable | |
|---|---|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | the collector's base URL; set, spans are exported |
| `OTEL_EXPORTER_OTLP_HEADERS` | headers to send, such as a vendor's API key (`x-honeycomb-team=…`) |
| `OTEL_SERVICE_NAME` | the service's name; by default the crate's |
| `OTEL_TRACES_SAMPLER`, `OTEL_TRACES_SAMPLER_ARG` | sampling, such as `parentbased_traceidratio` and `0.1` |
| `OTEL_RESOURCE_ATTRIBUTES` | more attributes, such as `deployment.environment=prod` |

Spans are batched and sent from a thread of their own; those still
buffered are sent when the server stops. Any backend that takes OTLP works
(Jaeger, Grafana Tempo, Honeycomb, Datadog, an OpenTelemetry Collector).

## Metrics

Set `METRICS_ADDR` (`0.0.0.0:9090` in a container) and `/metrics` is served
there, a listener of its own, so the API's port never shows it; keep that
port private and let Prometheus scrape it. What is measured
(`src/shared/metrics.rs`):

| Metric | Kind | Labels |
|---|---|---|
| `http_requests_total` | counter | `method`, `route` (`unmatched` for a path no route matched), `status` |
| `http_request_duration_seconds` | histogram | `method`, `route` |
| `db_pool_connections` | gauge | `state`: `idle`, `in_use` |
| `db_pool_max_connections` | gauge | |
| `jobs_total` | counter | `kind`, `outcome`: `done`, `failed` |
| `job_duration_seconds` | histogram | `kind` |
| `jobs_queued` | gauge | `state`: `due`, `scheduled`, `dead` |
| `auth_logins_total` | counter | `outcome`: `success`, `two_factor`, `bad_credentials`, `locked`, `forbidden` |
| `auth_lockouts_total` | counter | |

Alerts worth having from the start:

- the 5xx rate: `sum(rate(http_requests_total{status=~"5.."}[5m])) > 0.1`
- latency: the p95 of `http_request_duration_seconds` above your budget
- dead jobs: `jobs_queued{state="dead"} > 0`
- a queue falling behind: `jobs_queued{state="due"}` growing for 10 minutes
- the pool exhausted: `db_pool_connections{state="in_use"}` at
  `db_pool_max_connections`
- credential stuffing: `rate(auth_logins_total{outcome="bad_credentials"}[5m])`
  far above its usual level, `auth_lockouts_total` rising

## Locally: Jaeger, Prometheus, Grafana

```console
$ OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4318 docker compose --profile observability up --build
```

Jaeger at http://localhost:16686 shows the traces, Prometheus at
http://localhost:9091 scrapes `/metrics`, and Grafana at
http://localhost:3000 opens the *API* dashboard
(`deploy/compose/grafana/dashboards/api.json`): requests and latency by
route, server errors, the job queue, the pool and sign-ins.

## Your own spans and metrics

`#[tracing::instrument]` on a service method, or `tracing::info_span!`,
nests under the request's span and is exported with it. A metric is one
line anywhere, `metrics::counter!("orders_placed_total").increment(1)`,
exported with the others. Keep labels to a few known values: a label per
user or per id is a series per user, and the bill follows. To send metrics
over OTLP instead of serving `/metrics`, see the
[recipe](recipes/otel-metrics.md).
