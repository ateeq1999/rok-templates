# Recipe: metrics over OTLP

The templates serve their metrics as Prometheus text on `METRICS_ADDR`
([Observability](../observability.md#metrics)). Some backends want metrics
pushed over OTLP instead: Grafana Cloud's OTLP gateway, Honeycomb,
Datadog's OTLP intake, an OpenTelemetry Collector you already run.

## Without changing the code (recommended)

Let an OpenTelemetry Collector scrape `/metrics` and push it on. The API
does not change, and the Collector handles batching, retries and
credentials. Run it beside each task as a sidecar, or as one per cluster
that scrapes them all.

```yaml
# otel-collector.yaml
receivers:
  prometheus:
    config:
      scrape_configs:
        - job_name: api
          scrape_interval: 15s
          static_configs:
            - targets: ["localhost:9090"]   # METRICS_ADDR
exporters:
  otlphttp:
    endpoint: https://otlp.example.com
    headers:
      authorization: "Bearer ${env:OTLP_TOKEN}"
processors:
  batch: {}
service:
  pipelines:
    metrics:
      receivers: [prometheus]
      processors: [batch]
      exporters: [otlphttp]
```

The same Collector can take the traces too. Set
`OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318` and add an `otlp`
receiver and a `traces` pipeline, so one process holds the backend's
credentials.

## In the process

To send metrics without a Collector, replace the recorder. The
`metrics::counter!` and `histogram!` calls in `shared/metrics.rs` and in
your features stay as they are; what changes is the recorder that `install`
sets.

1. Add the OpenTelemetry SDK's `metrics` feature to `opentelemetry`,
   `opentelemetry_sdk` and `opentelemetry-otlp`, and a bridge from the
   `metrics` facade to OpenTelemetry (search crates.io for
   `metrics-exporter-opentelemetry`; check that it targets the
   `opentelemetry` version in `Cargo.toml`).
2. In `shared/metrics.rs`, build an OTLP metric exporter
   (`opentelemetry_otlp::MetricExporter::builder().with_http()`), a
   `SdkMeterProvider` with a periodic reader over it, and install the
   bridge's recorder over that provider in place of `PrometheusBuilder`.
3. Install it when `OTEL_EXPORTER_OTLP_ENDPOINT` is set instead of
   `METRICS_ADDR`, and shut the provider down beside the tracer in
   `Telemetry::shutdown`, so the last interval is sent.
4. Histograms: `http_request_duration_seconds`'s buckets are set in
   `install`. Set them again as a view on the provider, or the SDK's
   defaults apply.

Keep the metric names. The alerts and the Grafana dashboard
(`deploy/compose/grafana/dashboards/api.json`) query them.
