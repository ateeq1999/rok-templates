// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Logs and traces. [`init`] installs the one `tracing` subscriber
//! `main.rs` starts with: the log lines (`LOG_FORMAT`, `pretty` or `json`,
//! filtered by `RUST_LOG`) and, when `OTEL_EXPORTER_OTLP_ENDPOINT` is set,
//! every span exported over OTLP/HTTP to that collector. Unset, nothing is
//! exported and nothing is connected to.
//!
//! The exporter reads the standard `OTEL_*` variables itself
//! (`OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_EXPORTER_OTLP_HEADERS`,
//! `OTEL_TRACES_SAMPLER`, `OTEL_RESOURCE_ATTRIBUTES` …); `config.rs` only
//! decides whether to export. A request's span continues the caller's
//! trace when it sends a W3C `traceparent` ([`remote_parent`]).
//!
//! Framework-free, so both templates share it; each template's
//! `app/middleware.rs` opens the request span.

use std::fmt;

use opentelemetry::propagation::{Extractor, TextMapPropagator};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

/// How log lines are written (`LOG_FORMAT`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// `pretty`: one readable line per event, for a terminal. The default.
    #[default]
    Pretty,
    /// `json`: one JSON object per event, for a log collector.
    Json,
}

/// What [`init`] needs, read by `config::log_settings` before anything else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogSettings {
    /// `RUST_LOG`.
    pub filter: String,
    /// `LOG_FORMAT`.
    pub format: LogFormat,
    /// Whether `OTEL_EXPORTER_OTLP_ENDPOINT` is set: spans are exported.
    pub export_traces: bool,
    /// `OTEL_SERVICE_NAME`; unset, the crate's name names the service.
    pub service_name: Option<String>,
}

/// Tracing couldn't start: the exporter couldn't be built, or a subscriber
/// was installed already.
#[derive(Debug)]
pub struct TelemetryError(String);

impl fmt::Display for TelemetryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "tracing could not start: {}", self.0)
    }
}

impl std::error::Error for TelemetryError {}

/// The running telemetry; [`Telemetry::shutdown`] it when the server stops,
/// so the spans still buffered are sent.
#[derive(Debug)]
pub struct Telemetry {
    provider: Option<SdkTracerProvider>,
}

impl Telemetry {
    /// Whether spans are exported.
    #[must_use]
    pub fn exports(&self) -> bool {
        self.provider.is_some()
    }

    /// Sends the spans still buffered and stops the exporter. A failure is
    /// logged, never fatal: the server has stopped already.
    pub fn shutdown(self) {
        if let Some(provider) = self.provider
            && let Err(error) = provider.shutdown()
        {
            tracing::warn!(%error, "the trace exporter did not shut down cleanly");
        }
    }
}

/// Installs the subscriber `settings` describe, as the process's one.
pub fn init(settings: &LogSettings) -> Result<Telemetry, TelemetryError> {
    let provider = if settings.export_traces {
        Some(provider(settings)?)
    } else {
        None
    };
    let lines = match settings.format {
        LogFormat::Pretty => tracing_subscriber::fmt::layer().boxed(),
        LogFormat::Json => tracing_subscriber::fmt::layer().json().boxed(),
    };
    let spans = provider.as_ref().map(|provider| {
        tracing_opentelemetry::layer().with_tracer(provider.tracer(env!("CARGO_PKG_NAME")))
    });
    tracing_subscriber::registry()
        .with(lines)
        .with(spans)
        .with(EnvFilter::new(&settings.filter))
        .try_init()
        .map_err(|error| TelemetryError(error.to_string()))?;
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
    Ok(Telemetry { provider })
}

/// The tracer provider: spans batched and sent over OTLP/HTTP, the service
/// named `OTEL_SERVICE_NAME` or after the crate.
fn provider(settings: &LogSettings) -> Result<SdkTracerProvider, TelemetryError> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .build()
        .map_err(|error| TelemetryError(error.to_string()))?;
    let name = settings
        .service_name
        .clone()
        .unwrap_or_else(|| env!("CARGO_PKG_NAME").to_owned());
    Ok(SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::builder().with_service_name(name).build())
        .build())
}

/// The trace a request continues: the W3C `traceparent` and `tracestate`
/// it carries, if any. Set it as the request span's parent
/// (`OpenTelemetrySpanExt::set_parent`); without them, the span starts a
/// trace of its own.
#[must_use]
pub fn remote_parent(
    traceparent: Option<&str>,
    tracestate: Option<&str>,
) -> opentelemetry::Context {
    TraceContextPropagator::new().extract(&TraceHeaders {
        traceparent,
        tracestate,
    })
}

/// Records the caller on the current span, the request's: what
/// `shared/auth.rs` calls once it knows who is asking.
pub fn record_user(id: impl fmt::Display) {
    tracing::Span::current().record("user_id", tracing::field::display(id));
}

/// The two headers [`remote_parent`] reads.
struct TraceHeaders<'a> {
    traceparent: Option<&'a str>,
    tracestate: Option<&'a str>,
}

impl Extractor for TraceHeaders<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        match key {
            "traceparent" => self.traceparent,
            "tracestate" => self.tracestate,
            _ => None,
        }
    }

    fn keys(&self) -> Vec<&str> {
        vec!["traceparent", "tracestate"]
    }
}

#[cfg(test)]
mod tests {
    use opentelemetry::trace::TraceContextExt;

    use super::*;

    #[test]
    fn a_request_continues_the_trace_its_traceparent_names() {
        let parent = remote_parent(
            Some("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
            None,
        );
        let span = parent.span();
        let context = span.span_context();
        assert!(context.is_remote());
        assert_eq!(
            context.trace_id().to_string(),
            "4bf92f3577b34da6a3ce929d0e0e4736"
        );
        assert!(!remote_parent(None, None).span().span_context().is_valid());
        assert!(
            !remote_parent(Some("garbage"), None)
                .span()
                .span_context()
                .is_valid()
        );
    }
}
