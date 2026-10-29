# axum-api

A complete axum API made by [`rok sqlc`](https://github.com/ateeq1999/rok-data)
from one schema: the `users` and `audit_log` part of the axum starter's
OpenAPI document (`tests/spec/openapi.json`), with the same wire standard,
routes and schemas. `tests/contract.rs` checks the document this application
serves against that one, operation by operation. It is the actix template
(`rok sqlc init --template actix`) on axum 0.8: the same schema, queries,
services, security and mail code, with axum handlers, routes and
extractors.

Start one with:

```sh
rok sqlc init --template axum my-api
cd my-api
rok sqlc g
```

## Running it

You need PostgreSQL and Rust 1.94 or newer.

1. Create a database and apply `migrations/` to it — `rok migrate up` (the
   workspace CLI), or `psql -f` on each `migrations/*.up.sql` in order (the
   `.down.sql` of each rolls it back).
2. Set `DATABASE_URL`, `JWT_SECRET`, at least 32 bytes (`openssl rand
   -base64 48`), and `TOTP_ENCRYPTION_KEY`, 32 bytes in base64 (`openssl
   rand -base64 32`). `.env.example` lists every setting; `src/config.rs` reads
   them from the environment.
3. `cargo run`. The API is under `/api/v1`, health at `/health/live` and
   `/health/ready`, the OpenAPI document at `/api-docs/openapi.json` and
   Swagger UI at `/docs/`. Ctrl+C or `SIGTERM` stops it gracefully:
   requests in flight finish, then the job worker stops.

## Jobs

Background work runs on PostgreSQL itself (`src/shared/jobs/`, the same in
both templates): no other datastore. A flow writes a job on its own
transaction, so a rolled-back flow enqueues nothing; the job worker that
runs beside the server claims it (`FOR UPDATE SKIP LOCKED`: two workers,
in this process or another, never run the same job), runs it and deletes
it. A failure retries after 30s, 1m, 2m, 4m…; after its kind's last
attempt (5 by default) the job is `dead`, kept in `jobs` with its
`last_error`. Mail is one such job (`mail.send`).

Adding one takes four steps:

1. A payload: `#[derive(Serialize, Deserialize)] pub struct SendInvoice {
   pub order: OrderId }`.
2. `impl Job for SendInvoice` (`#[async_trait]`): its `KIND`
   (`"billing.send_invoice"`), the `Context` it runs with (a service), and
   `async fn run(self, billing: &BillingService)`. `MAX_ATTEMPTS`,
   `TIMEOUT` (5 minutes) and `on_dead` (what a dead job's payload keeps)
   have defaults.
3. Register it in `app/run.rs`'s `worker`:
   `.register::<SendInvoice>(state.billing.clone())`.
4. Enqueue it on the flow's transaction, from the service:
   `state.queue.enqueue(&mut tx, &SendInvoice { order })`, then
   `queue.commit(tx)` (commits, and wakes the worker at once).
   `enqueue_with` takes a `delay` and a `unique_key` (no second job with
   that key while one exists).

Delivery is at least once: a crash after a job ran but before it was
deleted runs it again, so make each job safe to repeat. A running job
holds one pool connection, so keep `JOBS_CONCURRENCY` (4) well under
`DB_MAX_CONNECTIONS`. A job's span continues the trace of the request that
enqueued it, and `/metrics` counts attempts (`jobs_total`) and the queue
(`jobs_queued`). In tests, `Worker::run_due()` runs the due jobs inline,
without sleeping (`tests/jobs.rs`).

### Scheduled jobs

A job on a clock is one line in `app/run.rs`'s `schedule`:
`.cron::<NightlyReport>("0 3 * * *")?` enqueues `NightlyReport::default()`
at every tick of the pattern (`minute hour day month weekday`, UTC; the
kind must be registered in `worker` too). The upload sweep is the first:
`media.sweep_uploads`, every 5 minutes. Every process running jobs runs
the schedule, and each tick runs once however many there are: the one
transaction that moves the kind's `last_tick` forward (`job_schedules`)
enqueues it. A tick missed while nothing ran (a deploy) runs once when a
process is back; a new schedule waits for its first tick rather than
running at once. A pattern that doesn't parse stops the start-up. Tests
call `Schedule::tick_at(pool, queue, now)` with the clock they want
(`tests/schedule.rs`).

### Processes: `APP_ROLE`

One image runs any part: `APP_ROLE=all` (the default) serves the API and
runs the job worker and the schedule; `web` serves the API only, so the
jobs it enqueues run elsewhere; `worker` runs the jobs and the schedule and
answers only `/health/live` and `/health/ready` on `BIND_ADDR`, for the
platform's health check. Scale `web` with traffic and `worker` with the
queue (`jobs_queued`).

## Logs and traces

`src/shared/telemetry.rs` (the same in both templates) starts tracing
before anything else, from `RUST_LOG`, `LOG_FORMAT` and the `OTEL_*`
variables:

- **Log lines.** `LOG_FORMAT=pretty` (the default) for a terminal,
  `LOG_FORMAT=json` for a log collector: one JSON object per event.
  `RUST_LOG` filters them (`info`, `debug,sqlx=warn`).
- **The request span.** Every request runs in a span carrying its method,
  its route (the matched pattern, `/api/v1/users/{id}`), its request id
  (also answered as `x-request-id`) and, once `shared/auth.rs` knows it,
  the caller's `user_id`. Everything logged while serving it carries them.
- **Trace export.** Set `OTEL_EXPORTER_OTLP_ENDPOINT` (such as
  `http://localhost:4318`) and every span is exported over OTLP/HTTP to
  that collector; a request sending a W3C `traceparent` continues the
  caller's trace. The exporter reads the other standard variables itself
  (`OTEL_EXPORTER_OTLP_HEADERS` for an API key, `OTEL_TRACES_SAMPLER`,
  `OTEL_RESOURCE_ATTRIBUTES`); `OTEL_SERVICE_NAME` names the service, by
  default the crate. Unset, nothing is exported or connected to. The spans
  still buffered are sent when the server stops.
- **Your own spans.** `#[tracing::instrument]` on a service method, or
  `tracing::info_span!`, nests under the request's span and is exported
  with it; no other wiring.

## Metrics

Set `METRICS_ADDR` (`0.0.0.0:9090` in a container, `127.0.0.1:9090` on
your machine) and `/metrics` is served there in Prometheus's text format,
on a listener of its own: the API's port never shows it, so keep the
metrics port off the internet and let Prometheus scrape it. Unset, nothing
is recorded or listens. `src/shared/metrics.rs` (the same in both
templates) lists what is measured:

- `http_requests_total` and `http_request_duration_seconds`, by method,
  route (the matched pattern, so `/api/v1/users/{id}` is one series; a
  path no route matched is `unmatched`) and status;
- `db_pool_connections{state="idle"|"in_use"}` and
  `db_pool_max_connections`, read every 15 seconds;
- `auth_logins_total{outcome}` (`success`, `two_factor`,
  `bad_credentials`, `locked`, `forbidden`) and `auth_lockouts_total`.

Add your own with the `metrics` macros anywhere
(`metrics::counter!("orders_placed_total").increment(1)`). Keep a label to
a few known values: a label per user or per id is a series per user.

## Security

Every answer passes through `src/app/middleware.rs`, over what
`src/shared/security/headers.rs` decides (the same in both templates):

- **Headers.** `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
  `Content-Security-Policy: frame-ancestors 'none'` and `Referrer-Policy:
  no-referrer` on every answer, and `Strict-Transport-Security` once
  `APP_URL` is `https://`. A header a handler set itself is kept.
- **CORS.** Unset, `CORS_ALLOWED_ORIGINS` allows no other origin. Set it to
  your front ends (`https://app.example.com,http://localhost:5173`), or `*`;
  a request from another origin gets no `Access-Control-Allow-Origin`.
- **Panics.** A handler that panics is answered `500 internal_error`, and
  the panic is logged, instead of the connection being dropped.
- **Rate limits.** `AUTH_RATE_LIMIT` (20 a minute per IP) guards `/auth`;
  `GLOBAL_RATE_LIMIT` (off by default) guards every route but health, so a
  probe is never refused. Both count per process.
- **Rotating `JWT_SECRET`.** Set the new secret as `JWT_SECRET` and the old
  one as `JWT_SECRET_PREVIOUS`, wait one `JWT_TTL`, then remove it: nobody
  is signed out. The links the API signs itself (for a store that can't
  sign its own) use a key derived from `JWT_SECRET`, so those handed out
  before the switch stop working
  (they last minutes).
- **Dependencies.** `deny.toml` is `cargo deny check`'s policy: no known
  advisory, licenses from an allow list, crates only from crates.io. The
  advisories it ignores are listed there, each with why it can't reach this
  project; review them when you update.

## Accounts

The account flows are hand-written features, yours to change:
`src/features/auth/`, `src/features/two_factor/` and
`src/features/account/`. The engine's guide
documents each one, the settings, sending real mail and adding a flow
([Accounts and sign-in](https://github.com/ateeq1999/rok-data/blob/main/crates/rok-data-sqlc/docs/guide/auth.md)).

- **Sign-up.** `POST /api/v1/auth/register` (`{"email", "password"}`, 8 to
  128 characters) creates the account and mails a verification link. Its
  token goes to `POST /api/v1/auth/email/verify`; a signed-in user asks for
  a new link with `POST /api/v1/auth/email/verification/resend`. With
  `MAIL_TRANSPORT=log` (the default) the mail is logged, and
  `RUST_LOG=debug` shows its body.
- **Sign-in.** `POST /api/v1/auth/login` answers a session token. Five
  wrong passwords in a row lock an account for 15 minutes, and an
  unverified email is a `403` while `REQUIRE_VERIFIED_EMAIL=true`.
- **Two-factor sign-in.** `POST /api/v1/auth/2fa/setup` answers a new
  authenticator secret (base32, its `otpauth://` URI and a QR code), and
  `POST /api/v1/auth/2fa/enable` (`{"code"}`) turns two-factor on with a
  code from the app, answering ten single-use recovery codes once. Then the
  right password answers `{"requires_totp": true, "pending_token"}`, and
  `POST /api/v1/auth/2fa/verify` (`{"pending_token", "code"}`, within 5
  minutes) trades it, with an app code or a recovery code, for the session.
  A code is accepted once; wrong codes count toward the lock. `POST
  /api/v1/auth/2fa/disable` (`{"current_password"}`) turns it off and
  signs every session out. Both changes are mailed. The secret is stored
  encrypted (AES-256-GCM under `TOTP_ENCRYPTION_KEY`, bound to the
  account), the recovery codes as HMAC-SHA256 under a key derived from it.
- **Passwords.** `POST /api/v1/auth/password/forgot` (`{"email"}`, always
  `202`) mails a link valid an hour, and `POST /api/v1/auth/password/reset`
  (`{"token", "new_password"}`) sets the new password and lifts any lock. A
  signed-in user changes theirs with `POST /api/v1/auth/password/change`
  (`{"current_password", "new_password"}`) and gets a notice by mail. Both
  sign every older session out. A reset also revokes every API key and any
  pending two-factor sign-in (it recovers an account someone else may have
  had); a change keeps the keys. `BREACH_CHECK=true` refuses passwords Have
  I Been Pwned knows.
- **Email change.** `POST /api/v1/auth/email/change`
  (`{"current_password", "new_email"}`, `202`) mails a notice to the old
  address and, when the new one is free, a link to it;
  `POST /api/v1/auth/email/change/confirm` (`{"token"}`) moves the account.
- **Invitations.** An admin's `POST /api/v1/auth/invitations` (`{"email",
  "display_name"?, "role"?}`) creates an account with no password, records
  it in the audit log, and mails a link valid 7 days; the invitee sets a
  password with it through `POST /api/v1/auth/password/reset`.
- **The account.** `GET /api/v1/auth/me` and `GET`/`PATCH
  /api/v1/users/me` are the signed-in user's own.
- **Profile photo.** `PUT /api/v1/users/me/avatar` takes a raw JPEG, PNG
  or WebP image of at most 2 MiB (`413` past it; anything undecodable, or
  over 8192×8192 pixels, is a `422`), stores a 256×256 JPEG of its centre
  (re-encoded, so no EXIF survives) and deletes the previous one;
  `DELETE /api/v1/users/me/avatar` removes it. Every user response
  answers `avatar_url`, `/api/v1/avatars/<file>` or `null`, which anyone
  may fetch, cacheable for good (a new photo gets a new name).
- **API keys.** `src/features/api_keys/`: `POST /api/v1/api-keys`
  (`{"name", "scope"?, "expires_in_days"?}`) answers a `rok_…` key once
  (only its prefix and SHA-256 are stored); `GET` lists the caller's
  active keys and `DELETE /api/v1/api-keys/{id}` revokes one. A key works
  wherever a session does, as `Authorization: Bearer rok_…` or
  `X-API-Key`, except under `/auth` and `/api-keys` (`403`); a `read`
  key only for `GET`, `HEAD` and `OPTIONS`. A key acts with its owner's
  role, an admin's included. At most 25 active keys (`409 conflict` past
  it); `expires_in_days` is 1 to 365.

Every `/auth` route is limited to `AUTH_RATE_LIMIT` requests a minute per
IP (`429` past it). A flow refusing a well-formed request answers `400`
with `invalid_token`, `wrong_password` or `breached_password`; a body of the
wrong shape answers `422`.

## Uploads

`/api/v1/media` holds each signed-in user's files (the hand-written
`media` feature):

- `POST /api/v1/media` takes the raw file as its body (its `Content-Type`
  is ignored), at most `MEDIA_MAX_UPLOAD_BYTES` (10 MiB; `413` past it).
  The type is detected from the bytes and must be one of
  `MEDIA_ALLOWED_TYPES` (JPEG, PNG, WebP, GIF and PDF; else `415`).
  `?filename=` or the `X-Filename` header names the file (255 characters
  at most). It answers `201` with the file's metadata and its `url`,
  `/api/v1/media/{id}/content`.
- `GET /api/v1/media` lists the caller's files, newest first
  (`?page=&per_page=`); `GET` and `DELETE /api/v1/media/{id}` read or
  delete one.
- `GET /api/v1/media/{id}/content` answers the bytes with the detected
  `Content-Type`, `X-Content-Type-Options: nosniff`, and
  `Content-Disposition: attachment` for anything but an image.

Another user's file is a `404`. The upload route has its own `DefaultBodyLimit`,
in place of the 2 MiB `router` sets. The bytes go through the
`Storage` port in `src/shared/storage/` (put, get and delete by key, the
content type kept), over `object_store`, chosen by `STORAGE_URL`:
`file://./storage` (the default, a `storage/` directory where the server
runs), `s3://<bucket>` (the credentials, region and `AWS_ENDPOINT` come
from the `AWS_*` variables) or `memory://`, opened by `AppState::new`
(an unopenable store fails startup). The tests configure `memory://`.

### Direct uploads

A client can send a file straight to the store instead of through the API
(engine ADR 0024):

1. `POST /api/v1/media/uploads` with `{"content_type", "size_bytes",
   "filename"?}` checks the size (`413`) and type (`415`) up front and
   answers `201` with an upload ticket: `media_id`, `method` (`PUT`),
   `upload_url`, the `headers` to send and `expires_at`
   (`MEDIA_UPLOAD_URL_TTL_SECS`, 15 minutes).
2. The client `PUT`s the bytes to `upload_url`, with those headers and no
   `Authorization`.
3. `POST /api/v1/media/{id}/complete` checks the stored bytes: their size
   must be the one declared and their detected type the one declared, or
   the object is deleted and the answer is `422` (`409` while nothing is
   stored yet). It answers the file's metadata; until then the file is in
   no listing.

`GET /api/v1/media/{id}/url` answers `{url, expires_at}`, a download URL
that needs no credentials (`MEDIA_DOWNLOAD_URL_TTL_SECS`, 5 minutes).

An `s3://` store signs these URLs itself (SigV4); set
`STORAGE_PUBLIC_ENDPOINT` when clients reach it at another address than
the API does (MinIO in Docker). A `file://` or `memory://` store can't
sign, so the API signs them, under a key derived from `JWT_SECRET`, and
serves them at `/api/v1/storage/{token}`. Uploads never completed are
deleted every 5 minutes, by the scheduled `media.sweep_uploads` job (see
Jobs).

## The server

`src/app/` is the only wiring: `state.rs`, `router.rs`, `run.rs` and
`ports.rs`, one part each. `AppState` (`#[derive(FromRef)]`)
holds the pool, the generated features' `Services`, the JWT keys, the mail
outbox and templates, the breach check and the rate limit; a handler takes
any of them with `State<...>`. `router(state)` merges health, Swagger UI
and every feature's `routes()`, under tower-http's tracing, request ids
(`x-request-id`), the security headers and CORS (see Security), a panic
answered `500`, `GLOBAL_RATE_LIMIT`, a 30-second
timeout (`408`) and a 2 MiB body limit (`413`). `serve` runs it with the
job worker beside it (`tokio::spawn`), both stopped by the same shutdown
signal.

Every other route but health needs `Authorization: Bearer <token>`: an HS256
JWT signed with `JWT_SECRET`, with `sub` (the user's id), `role` (`user`
or `admin`), `tv` (the account's `token_version`) and `exp`.
`src/shared/auth.rs`'s `AuthUser` and `AdminUser` verify it, then check
the account on every request (`queries/session.sql`): a disabled or
deleted account, or a token issued before the account's `token_version`
was bumped, is a `401`, and the role is read from the database, so a role
change applies at once.

- `src/shared/security/` holds argon2id password hashing, single-use
  tokens (only their SHA-256 is stored), the JWT, the breach check and
  two-factor's TOTP codes, secret encryption and recovery codes, and the
  single-use token lifecycle (`auth_token.rs`).
- `src/shared/mail/` sends mail through an outbox: a flow writes its
  message as a `mail.send` job on its own transaction, and the job worker
  delivers it (SMTP, or the log), retrying with backoff; after five
  failures the job is `dead`, and loses its bodies and the link in them.
- `src/shared/rate_limit/` is `RateLimit`, a tower `Layer` (`mod.rs`) over
  `governor` (`limiter.rs`, the same in both templates): `AUTH_RATE_LIMIT` requests a minute per client IP, then
  `429 too_many_requests` with `Retry-After`. The IP is the peer, or with
  `TRUST_PROXY=true` the last `X-Forwarded-For` address. Wrap routes in
  `state.rate_limit.clone()`.

## Dependencies

The template's `Cargo.toml` has them. Starting from a package of your own
(`init --template` keeps an existing `Cargo.toml`), add these:

```toml
[dependencies]
aes-gcm = { version = "0.10", default-features = false, features = ["aes", "alloc"] }
argon2 = "0.6"
async-trait = "0.1"
axum = { version = "0.8", features = ["json", "macros", "query"] }
base64 = "0.22"
bytes = "1"
chrono = { version = "0.4", features = ["serde"] }
croner = "4"
governor = { version = "0.10", default-features = false, features = ["std"] }
hmac = "0.13"
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }
infer = { version = "0.22", default-features = false }
jsonwebtoken = { version = "11", features = ["rust_crypto"] }
lettre = { version = "0.11", default-features = false, features = ["builder", "hostname", "smtp-transport", "tokio1-rustls-tls"] }
metrics = "0.24"
metrics-exporter-prometheus = { version = "0.18", default-features = false, features = ["http-listener"] }
object_store = { version = "0.13", features = ["aws"] }
opentelemetry = { version = "0.32", default-features = false, features = ["trace"] }
opentelemetry-otlp = { version = "0.32", default-features = false, features = ["trace", "http-proto", "reqwest-blocking-client", "reqwest-rustls"] }
opentelemetry_sdk = { version = "0.32", default-features = false, features = ["trace"] }
qrcode = { version = "0.14", default-features = false, features = ["svg"] }
rand = "0.10"
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls"] }
rok-data = { version = "0.4", features = ["chrono", "json", "uuid"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_path_to_error = "0.1"
sha1 = "0.11"
sha2 = "0.11"
sqlx = { version = "0.9", features = ["runtime-tokio", "postgres", "derive", "uuid", "chrono", "json"] }
subtle = "2"
thiserror = "2"
tokio = { version = "1", features = ["macros", "net", "rt", "rt-multi-thread", "signal", "sync", "time"] }
totp-rs = { version = "5", features = ["otpauth"] }
tower = { version = "0.5", features = ["util"] }
tower-http = { version = "0.6", features = ["catch-panic", "cors", "request-id", "timeout", "trace"] }
tracing = "0.1"
tracing-opentelemetry = { version = "0.33", default-features = false }
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
utoipa = { version = "5", features = ["chrono", "uuid"] }
utoipa-swagger-ui = { version = "9", features = ["axum", "vendored"] }
uuid = { version = "1", features = ["v7", "serde"] }
zeroize = "1"

[dev-dependencies]
http-body-util = "0.1"

[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
print_stdout = "deny"

[workspace]
```

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
DATABASE_URL=postgres://... cargo test   # each test makes and drops its own schema
rok sqlc g --check                       # the generated code matches the schema
rok sqlc check-arch                      # the layout and rules of AGENTS.md
```

Without `DATABASE_URL`, the API tests print a note and pass; the contract
and rate-limit tests need no database. The lints in `Cargo.toml` deny
`unwrap`, `expect`, `panic`, `todo` and leftover debugging.

## What is generated, and what is yours

`AGENTS.md` and `src/features/README.md` explain the layout. In short:

- **Generated** (first line `@generated`; `rok sqlc g` rewrites them, never
  edit them): `src/db/` (the data layer), `src/shared/{health,pagination,
  response,tenant,validation}.rs`, and every file of `src/features/` but
  `mod.rs` and each feature's `service.rs`.
- **Yours** (first line `Scaffolded once`, or no header): `rok-data-sqlc.toml`,
  `schema/` (the truth: the `.sql` files `rok sqlc g` reads), `queries/`,
  `src/{lib,main,config,error,openapi}.rs`, `src/app/`,
  `src/shared/{mod,auth}.rs`, `src/shared/rate_limit/`,
  `src/shared/security/`, `src/shared/mail/`, `src/shared/storage/`,
  `src/shared/thumbnail.rs`,
  `src/features/mod.rs`, the hand-written `src/features/{auth,account,media}/`,
  each feature's `service.rs` (business rules go there), `tests/`, and this README.
- **Generated**: `migrations/` is the history — `rok gen --migration` (the
  workspace CLI) stages its pairs and refreshes `.rok-schema.json`; never
  edit those files.

`queries/` holds hand-written SQL, one file per feature: `rok sqlc g` turns
`queries/auth.sql` into `src/db/queries/auth.rs`, one function per query.
Only the feature of that name calls it, whatever tables its queries touch;
`check-arch` reports another feature that does. A file named after no
feature is anyone's: `queries/session.sql` (the check `AuthUser` makes)
and `queries/queue.sql` (the job queue) serve `src/shared/`.

Grow the truth in `schema/`, stage the change with `rok gen --migration
<name>`, adjust `rok-data-sqlc.toml`, run `rok gen` (or `rok sqlc g`), and the
features follow.

## How the schema meets the target

- `role` is a PostgreSQL enum, so the schema `Role` is `user` or `admin`.
- `has_password` is a generated column (`password_hash IS NOT NULL`).
- `password_hash`, `totp_secret`, `totp_last_step`, `avatar_key`,
  `failed_logins`, `locked_until`, `token_version`, `updated_at` and
  `deleted_at` are hidden
  (`[api.hidden]`): no response carries them.
- The columns the server sets are managed (`[conventions]
  managed_columns`), so no request body takes them; a trigger keeps
  `updated_at` current.
- `deleted_at` is the soft-delete column: `DELETE /users/{id}` marks the row.
- `email` is immutable (`[api.immutable]`): set when an account is created,
  never by `PATCH /users/{id}`, which answers `422` when its body names it.
  An email changes through `POST /api/v1/auth/email/change` and the link
  it mails to the new address.
- `audit_log` is generated as the `audit_log_entry` module
  (`AuditLogEntryResponse`), served at `/audit-log`, read-only.
- `[api.search]` and `[api.sort]` give the users list its `q` and
  `UserSort` (`created_at`, `email`).
- `api_keys` is in the schema for a feature to come, `media` serves the
  hand-written `media` feature, and
  `auth_tokens` (single-use tokens), `totp_recovery_codes` (two-factor's
  recovery codes) for the account flows, and `jobs` (the job queue);
  none has a generated API (`[api] exclude`).
- `token_version` is bumped to sign an account out everywhere: a token
  carrying an older one is rejected.

`tests/contract.rs` lists the differences that remain, each with its
reason: `avatar_url` (uploads, on every `UserResponse`), and the
`?permanent=` and list `sort`/`order` parameters every generated API has.

## Shared with the actix template

The framework-independent files are the same in both templates, byte for
byte: `migrations/`, `queries/`, `src/shared/security/`, `src/shared/mail/`,
`src/shared/storage/`, `src/app/ports.rs`
and each feature's `dto.rs`, `model.rs`, `repo.rs`, `service.rs` and
`service_api.rs`, the hand-written `auth`, `account` and `media` included:
they differ only in their handlers and routes. This repository checks that
they stay so (`template_drift`).
