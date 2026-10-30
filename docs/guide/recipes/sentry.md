# Recipe: errors to Sentry

The templates log every server error and export traces over OTLP, so
Sentry is not needed to see failures. Add it when you want Sentry's
grouping, alerts and releases for errors. Sentry also takes OTLP traces,
so for traces alone, point `OTEL_EXPORTER_OTLP_ENDPOINT` at it
([Observability](../observability.md#traces)) and stop there.

## 1. The crates

```toml
sentry = { version = "0.4", default-features = false, features = ["backtrace", "contexts", "panic", "reqwest", "rustls"] }
sentry-tracing = "0.4"
```

Pin the minor version `cargo add sentry sentry-tracing` picks.

## 2. Start it first, with the DSN as a setting

In `main.rs`, before `telemetry::init`, start the client from
`SENTRY_DSN`. Unset, it does nothing, like the other capabilities. Keep
the guard alive until `main` returns: dropping it sends what is still
buffered.

<!-- not compiled: a recipe; adapt it to your main.rs -->
```rust
let _sentry = sentry::init(sentry::ClientOptions {
    dsn: std::env::var("SENTRY_DSN").ok().and_then(|dsn| dsn.parse().ok()),
    release: sentry::release_name!(),
    environment: std::env::var("APP_ENV").ok().map(Into::into),
    send_default_pii: false,
    ..Default::default()
});
```

`send_default_pii: false` keeps IP addresses and headers out of the events.
Leave it off: the `Authorization` header is a credential.

## 3. Send the error events

Add Sentry's layer to the subscriber, in `shared/telemetry.rs`'s `init`:

<!-- not compiled: a recipe; one more layer in telemetry::init -->
```rust
tracing_subscriber::registry()
    .with(lines)
    .with(spans)
    .with(sentry_tracing::layer())
    .with(EnvFilter::new(&settings.filter))
```

By default an `error!` event becomes a Sentry event, a `warn!` or `info!`
becomes a breadcrumb on the next one, and the rest is dropped. The
templates log every `500` once, at `error`, with its cause (`error.rs`), so
each server error reaches Sentry once, with the request span's fields: the
route, the request id and the `user_id`. A `4xx` is the caller's mistake
and is not logged as an error, so it stays out of Sentry.

## 4. Check it

Start with `SENTRY_DSN` set and trigger one server error, such as a route
that answers `Err(AppError::Internal(...))`. The event names the release
and holds the request id the caller was answered with (`x-request-id`).
Match the two when a user reports a failure.

The actix template is the same: the `main.rs` and `shared/telemetry.rs`
these steps change are the same in both.
