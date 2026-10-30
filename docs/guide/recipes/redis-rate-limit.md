# Recipe: a shared rate limit on Redis

`AUTH_RATE_LIMIT` and `GLOBAL_RATE_LIMIT` are counted in each process
([Security](../security.md#rate-limits)), so with three replicas a client
gets three times the limit. That is usually fine, since the limit's job is
to stop guessing, and three times 20 a minute still stops it. When it is
not fine, count in Redis.

## 1. The crate and the setting

```toml
redis = { version = "0.32", default-features = false, features = ["tokio-rustls-comp", "connection-manager"] }
```

Use the version `cargo add redis` picks. Read `REDIS_URL` in `config.rs`
like the other settings: required once the limit uses it, its password
left out of the start-up summary.

## 2. Count in Redis

A fixed one-minute window per IP: `INCR` a key named for the IP and the
minute, give it an expiry the first time, and refuse past the limit. Keep
`client_ip` from `shared/rate_limit/limiter.rs`, so `TRUST_PROXY` still
decides whose address counts.

<!-- not compiled: a recipe; the counting beside shared/rate_limit/limiter.rs -->
```rust
#[derive(Clone)]
pub struct SharedRateLimit {
    redis: redis::aio::ConnectionManager,
    per_minute: u32,
    scope: &'static str, // "auth", "global"
    trust_proxy: bool,
}

impl SharedRateLimit {
    pub async fn check(&self, ip: IpAddr) -> Result<(), AppError> {
        let now = chrono::Utc::now().timestamp();
        let window = now / 60;
        let key = format!("rate:{}:{ip}:{window}", self.scope);
        let (count,): (u32,) = redis::pipe()
            .atomic()
            .incr(&key, 1)
            .expire(&key, 120)
            .ignore()
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|error| AppError::Internal(error.to_string()))?;
        if count > self.per_minute {
            let retry_after_secs = u64::try_from(60 - now % 60).unwrap_or(1).max(1);
            return Err(AppError::TooManyRequests { retry_after_secs });
        }
        Ok(())
    }
}
```

A fixed window lets a client make up to twice the limit across a window's
edge. That is acceptable for a guard against guessing. For a smooth limit,
run the GCRA (what `governor` does in process) as a Lua script on Redis.

## 3. Wrap the routes

The in-process `RateLimit` checks synchronously, but this check awaits, so
wrap the routes in a middleware function. In axum:

<!-- not compiled: a recipe; replaces `.layer(limit.clone())` in the auth routes -->
```rust
async fn shared_limit(
    State(limit): State<SharedRateLimit>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let forwarded_for = request
        .headers()
        .get_all(FORWARDED_FOR)
        .iter()
        .filter_map(|value| value.to_str().ok());
    let ip = client_ip(forwarded_for, Some(peer.ip()), limit.trust_proxy);
    match limit.check(ip).await {
        Ok(()) => next.run(request).await,
        Err(refused) => refused.into_response(),
    }
}

// routes: .layer(axum::middleware::from_fn_with_state(limit, shared_limit))
```

In actix, `shared/rate_limit/mod.rs` is the middleware. Have its `call`
await the check inside the boxed future before calling the service.

## 4. Decide what happens when Redis is down

As written, a Redis failure is a `500`, so the sign-in routes go down with
Redis. The other choice is to let the request through and log a warning:
the API stays up, but unlimited. For `/auth`, a middle way is to fall back
to the in-process `RateLimit` and keep it as the second check. Whichever
you pick, test it by stopping Redis in `compose.yaml`.
