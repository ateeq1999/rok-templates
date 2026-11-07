# grpc-api

A tonic gRPC API made by [`rok sqlc`](https://github.com/ateeq1999/rok-data)
from the axum template's schema (workspace ADR 0030, engine ADR 0028): the
`users` and `audit_log` Resources, each a `.proto` under `proto/` and a
generated tonic service with unary `Get`, `List`, `Update`, `Delete` (as
`rok-data-sqlc.toml` allows each), over the same service trait the axum
handlers call. It shares the axum template's configuration, data layer,
jobs, mail, storage, telemetry, metrics, shutdown and security code, file
for file, and serves `grpc.health.v1.Health` and server reflection.

Start one with:

```sh
rok sqlc init --template grpc my-api
cd my-api
rok sqlc g
```

## Running it

You need PostgreSQL and Rust 1.94 or newer. You don't need `protoc`:
`build.rs` compiles `proto/` with [protox](https://crates.io/crates/protox),
a protobuf compiler in Rust, into the messages, the servers and the Rust
clients (`tonic-prost-build`).

1. Create a database and apply `migrations/` to it — `rok migrate up` (the
   workspace CLI), or `psql -f` on each `migrations/*.up.sql` in order. They
   are the axum template's: one database can serve both APIs.
2. Set `DATABASE_URL`, `JWT_SECRET` (the axum API's, at least 32 bytes) and
   `TOTP_ENCRYPTION_KEY` (32 bytes in base64). `.env.example` lists the
   settings; `src/config.rs` reads them from the environment.
3. `cargo run`. The server listens on `BIND_ADDR` (default
   `0.0.0.0:8080`). Ctrl+C or `SIGTERM` stops it gracefully: the health
   service answers `NOT_SERVING` for `SHUTDOWN_DELAY_SECS`, calls in flight
   finish, then the job worker stops.

```sh
grpcurl -plaintext localhost:8080 list
grpcurl -plaintext localhost:8080 grpc.health.v1.Health/Check
```

## The server

`src/app/routes.rs` builds what the server serves:

- the generated features' services (`features::routes`) and the
  hand-written `UserEventsService` (see [Your own RPCs](#your-own-rpcs)),
  behind the caller check of `src/app/auth.rs`;
- `grpc.health.v1.Health` (`tonic-health`): `SERVING` for the server
  (`""`), `NOT_SERVING` while it drains;
- server reflection (`tonic-reflection`, `grpc.reflection.v1` and
  `v1alpha`), describing `proto/` and the health service, so `grpcurl` and
  Postman call the services without the `.proto` files.

Health and reflection sit outside the caller check: they need no
credentials. grpc-web is off.

`APP_ROLE` says what a process runs: `all` (the server and the job worker),
`web` (the server) or `worker` (the jobs and the schedule, serving health
and reflection only).

## Who is calling

The generated services read the caller, an `AuthUser`
(`src/shared/auth.rs`, the axum template's type), from the request's
extensions (`shared/grpc.rs`): no caller is `UNAUTHENTICATED`, a caller
who is not an admin on an admin-only service (`[api.access]`)
`PERMISSION_DENIED`. `src/app/auth.rs` (a tower layer, since it reads the
database) puts it there, checking the metadata as the axum template's
extractor checks the headers:

- `authorization: Bearer <jwt>`: the axum API's session token, verified
  with the same `JWT_SECRET` (and `JWT_SECRET_PREVIOUS` while it is
  rotated out) and the same claims, then its account loaded: an expired
  or forged token, a deleted or disabled account, or a token older than
  the account's last `token_version` bump is `UNAUTHENTICATED`. The role
  comes from the database.
- `authorization: Bearer rok_…` or `x-api-key: rok_…`: an API key the axum
  API made. An unknown, revoked or expired key is `UNAUTHENTICATED`; a
  `read` key may call only the reads (`Get`, `List` and the example's
  `Subscribe`, the RPC names in `auth.rs`'s `READS`: add your own
  read-only RPCs there), anything else is `PERMISSION_DENIED`.
- Neither: the call reaches the service without a caller, and each
  generated service answers `UNAUTHENTICATED`.

```sh
grpcurl -plaintext -H "authorization: Bearer $TOKEN" \
  -d '{"id": "…"}' localhost:8080 grpc_api.v1.UsersService/Get
```

This API verifies credentials and never issues them: login, registration,
two-factor sign-in and making API keys stay on the HTTP API. Neither
template is multi-tenant, so the caller check sets no tenant; a
tenant-scoped service's `shared::grpc::tenant` reads the one you insert
beside the caller in `auth.rs`.

A rule beyond "signed in" or "admin" is a `Guard`
(`src/shared/security/guard.rs`, the axum template's): written once, and
checked in your own RPC with
`require::<PartnerAccess>(&caller(&request)?, &state, &path).await?`,
which gives its grant or answers `PERMISSION_DENIED`. The guide's
[Guards](https://github.com/ateeq1999/rok-templates/blob/main/docs/guide/guards.md)
page has a recipe.

## Errors

`src/error.rs`'s `AppError` is the axum template's, without the HTTP
answer; the generated `shared/grpc.rs` turns it into a status: not found
`NOT_FOUND`, a validation failure `INVALID_ARGUMENT` with the field errors
as `google.rpc.BadRequest` details, a unique or foreign-key violation
`ALREADY_EXISTS`, no credentials `UNAUTHENTICATED`, forbidden
`PERMISSION_DENIED`, anything else `INTERNAL` (logged, never sent).

## Your own RPCs

Write a `proto/<name>.proto` of your own beside the generated ones, in the
same package (`grpc_api.v1`), importing the generated messages
(`import "users.proto";`). `build.rs` compiles every `.proto` in
`proto/`; implement its service in a hand-written feature and add it in
`src/app/routes.rs`. Streaming is never generated.

The template carries one, as the example: `UserEventsService.Subscribe`
(`proto/user_events.proto`, the `user_events` feature), a server-streaming
RPC that sends an admin a `UserEvent` each time a user changes. It is a
read: a `read` API key may call it (`app/auth.rs`'s `READS`).

```sh
grpcurl -plaintext -H 'authorization: Bearer ...' \
  localhost:8080 grpc_api.v1.UserEventsService/Subscribe
```

- **Where the events come from.** PostgreSQL `LISTEN`/`NOTIFY` on the
  `user_events` channel, the payload a user's id:
  `NOTIFY user_events, '<user id>'`. `UserEventsService::publish` sends one
  (`repo::publish` on a transaction sends it on commit), and so may
  anything sharing the database: the axum API, a job, `psql`, or a trigger
  you add in a migration of your own, such as
  `PERFORM pg_notify('user_events', NEW.id::text)` after an update of
  `users`. The template adds no trigger: its migrations are the axum
  template's, byte for byte. The generated `users` service publishes
  nothing either; to publish from it, call `publish` from your own code
  or add that trigger.
- **What an event holds.** The id, and the user as it is when the event is
  sent (read once per subscriber), or no user once it is gone. A payload
  that is no id is logged and skipped.
- **One connection per subscriber.** Each subscription listens on a pool
  connection of its own (`sqlx::postgres::PgListener`) until the client
  hangs up, which drops it: size `DB_MAX_CONNECTIONS` for your
  subscribers, or fan one listener out to many in a feature of your own.
- **Slow subscribers and lost connections.** Events wait in a queue of 16;
  a subscriber that falls that far behind is ended with
  `RESOURCE_EXHAUSTED` rather than left to pile notifications up in
  PostgreSQL. A lost database connection ends the stream with
  `UNAVAILABLE`, since the notifications sent meanwhile are gone. Either
  way the client subscribes again and re-reads what it shows.
- **Shutdown.** A stream is a call in flight: the server waits for it up to
  `SHUTDOWN_GRACE_SECS`, then drops it.

## Dependencies

The template's `Cargo.toml` has them; the gRPC stack is pinned exactly,
since tonic's crates move together. Starting from a package of your own
(`init --template` keeps an existing `Cargo.toml`), add these:

```toml
[dependencies]
aes-gcm = { version = "0.10", default-features = false, features = ["aes", "alloc"] }
argon2 = "0.6"
async-trait = "0.1"
base64 = "0.22"
bytes = "1"
chrono = { version = "0.4", features = ["serde"] }
croner = "4"
hmac = "0.13"
infer = { version = "0.22", default-features = false }
jsonwebtoken = { version = "11", features = ["rust_crypto"] }
lettre = { version = "0.11", default-features = false, features = ["builder", "hostname", "smtp-transport", "tokio1-rustls-tls"] }
metrics = "0.24"
metrics-exporter-prometheus = { version = "0.18", default-features = false, features = ["http-listener"] }
object_store = { version = "0.13", features = ["aws"] }
opentelemetry = { version = "0.32", default-features = false, features = ["trace"] }
opentelemetry-otlp = { version = "0.32", default-features = false, features = ["trace", "http-proto", "reqwest-blocking-client", "reqwest-rustls"] }
opentelemetry_sdk = { version = "0.32", default-features = false, features = ["trace"] }
prost = "=0.14.4"
prost-types = "=0.14.4"
qrcode = { version = "0.14", default-features = false, features = ["svg"] }
rand = "0.10"
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls"] }
rok-data = { version = "0.9", features = ["chrono", "json", "uuid"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha1 = "0.11"
sha2 = "0.11"
sqlx = { version = "0.9", features = ["runtime-tokio", "postgres", "derive", "uuid", "chrono", "json"] }
subtle = "2"
thiserror = "2"
tokio = { version = "1", features = ["macros", "net", "rt", "rt-multi-thread", "signal", "sync", "time"] }
tokio-stream = "=0.1.19"
tonic = "=0.14.9"
tonic-health = "=0.14.9"
tonic-prost = "=0.14.9"
tonic-reflection = "=0.14.9"
tonic-types = "=0.14.9"
totp-rs = { version = "5", features = ["otpauth"] }
tower = "=0.5.3"
tracing = "0.1"
tracing-opentelemetry = { version = "0.33", default-features = false }
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
uuid = { version = "1", features = ["v7", "serde"] }
zeroize = "1"

[build-dependencies]
protox = "=0.10.0"
tonic-prost-build = "=0.14.9"

[dev-dependencies]
hyper-util = { version = "=0.1.21", features = ["tokio"] }
tower = { version = "=0.5.3", features = ["util"] }

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
```

Without `DATABASE_URL`, the tests that reach the data layer print a note and
pass; the health, reflection, caller and shutdown tests need no database.
They call the server over an in-process channel (`tests/support/mod.rs`),
the caller put in place by a stand-in for `app/auth.rs`.

## What is generated, and what is yours

- **Generated** (first line `@generated`; `rok sqlc g` rewrites them, never
  edit them): `proto/<table>.proto`, `src/db/`, `src/shared/{grpc,
  pagination,response,validation}.rs`, and every file of `src/features/`
  but `mod.rs`, each feature's `service.rs` and the hand-written
  `user_events` feature.
- **Yours**: `rok-data-sqlc.toml`, `schema/`, `queries/`, `build.rs`,
  `src/{lib,main,config,error}.rs`, `src/app/`, `src/shared/` but the
  generated files, `src/features/mod.rs`, each feature's `service.rs`,
  `src/features/user_events/` and `proto/user_events.proto`,
  `tests/`, and this README.
- **Generated**: `migrations/` is the history — `rok gen --migration`
  stages its pairs; never edit those files.

## How the schema meets the target

The schema and the `[api]` settings are the axum template's, with what gRPC
changes:

- every column of a message follows engine ADR 0028's mapping: ids and text
  `string`, times `google.protobuf.Timestamp`, a nullable column
  `optional`, `role` a nested enum whose first value is
  `ROLE_UNSPECIFIED = 0`;
- `List` pages by number (`page`, `per_page`) and answers `items`,
  `total`, `page` and `per_page`, as the HTTP list does;
- there is no OpenAPI document (`openapi` stays off), so the data layer's
  rows derive no schemas.

## Shared with the axum template

`config.rs`, `migrations/`, `schema/`, `queries/`, `src/shared/jobs/`,
`src/shared/mail/`, `src/shared/storage/`, `src/shared/security/`,
`src/shared/{telemetry,metrics,settings,shutdown}.rs`, the data layer's
`queries/`, `pool.rs`, `nullable.rs` and `schema.rs`, and each feature's
`model.rs`, `repo.rs`, `service.rs` and `service_api.rs` are the axum
template's, byte for byte. This repository checks that they stay so
(`template_drift`).
