# Changelog

All notable changes to `rok-templates` are documented here. The format
is [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and the
version numbers follow [Semantic
Versioning](https://semver.org/spec/v2.0.0.html). Release notes for a tag
are this file's `## [X.Y.Z]` section.

## [Unreleased]

**Added**

- Live updates over Topics. `shared/live` (axum, actix and the monorepo's
  API) publishes a notice on a feature's transaction,
  `live::publish(&mut tx, tenant, "orders.new", payload)`, as a PostgreSQL
  `NOTIFY`, so nothing is sent for a rollback; one listener per instance
  fans notices out to its subscribers through a bounded buffer, dropping
  one that falls behind. The `live` feature serves them:
  `POST /api/v1/live/tickets` gives a caller the `Subscriber` guard lets in
  a short-lived ticket to its tenant (403 otherwise), and
  `GET /api/v1/live?topics=…&ticket=…` streams the notices as server-sent
  events. The monorepo's web App gains `subscribe` and `useTopics`
  (`src/lib/live.ts`), which invalidate the queries a notice names, and its
  mobile App `LiveRepository`, a `Stream<Notice>`; both reconnect with a
  fresh ticket after a dropped connection. The guide's "Live updates" page
  has the rest.
- Prerendered pages in the monorepo's web App. `PRERENDERED` in
  `apps/web/vite.config.ts` (empty by default) lists the pages the build
  writes as finished HTML, e.g. a landing page at `/`; no route is
  discovered and no link followed, so signed-in routes always render in
  the browser. The SPA shell is now rendered at `/_shell` (a route that
  sends a visitor home) instead of `/`, so a listed page never replaces
  `_shell.html`; listing `/_shell` fails the build with an error naming
  it. The guide's monorepo page has a "Prerendered pages" section.
- Mail a feature words itself. `shared/mail` (every API template) gains
  `Templates::message(to, subject, paragraphs)`, worded like the account
  mails with each paragraph escaped in the HTML body, and
  `Mail::reply_to`: a field `SmtpMailer` sends as `Reply-To`, serde-defaulted
  so mail queued before it still decodes, and a builder, so a call reads
  `templates.message(to, subject, &paragraphs).reply_to(business_email)`.
- Guards: authorization rules written once. The security Capability's
  `Guard` trait (`shared/security/guard.rs`, a Shared file of every API
  template) names a rule's grant, its path parameters and its refusal;
  `Allowed<G>` (`shared/auth.rs`, axum, actix and the monorepo's API) runs
  it ahead of the body and hands the handler the grant
  (`Allowed(access): Allowed<PartnerAccess>`, `_: Allowed<Finance>`),
  answering `403` before the body is read. Under actix-web, which polls a
  handler's extractors together, `Allowed` holds the body back until the
  guard lets the caller in; `app::configure` registers the `AppState` as
  app data for it. `require::<G>` answers the same outside HTTP (the gRPC
  template). The guide's Guards page has a recipe in Karibu Pass's shape
  (a staff role, a member's role or staff at a partner from the path, and
  `PartnerAccess<Organizer>`), each rule one read in `queries/shared/`;
  a gate adds it to an axum project, generates it and drives it.
- Local setup that works on the first run. The APIs load `.env` with
  dotenvy (`=0.15.7`) in debug builds only (`config::load_env_file`; a
  release build reads only the real environment), and their test support
  does before reading `DATABASE_URL`. A `seed` binary (`cargo run --bin
  seed`: `app/seed.rs`, a Shared file, and `queries/seed.sql`) makes a
  verified admin and a verified ordinary user and prints new passwords for
  them; `--promote <email>` makes an existing account an admin.
  `REQUIRE_VERIFIED_EMAIL` stays `true`, and `default-run` keeps
  `cargo run` the server.

**Changed**

- The contract test (`tests/contract.rs` in axum, actix and the monorepo's
  API) allows additions to a covered operation: a new optional request
  property, a new optional query parameter and a new response property
  pass with no `KNOWN` entry, so a project grows the starter's routes
  instead of adding twins beside them. A removed or retyped property, a
  new required property or parameter and a changed success status still
  fail, naming the operation. The `?permanent=` and list `sort`/`order`
  entries leave `KNOWN`.
- `compose.yaml` publishes PostgreSQL on `${POSTGRES_PORT:-5432}`, and
  `.env.example` sets `POSTGRES_PORT` and builds `DATABASE_URL` from it,
  against compose's `app` database (the axum template's and the Monorepo
  API's named a stale `actix_api` one), with a local-development
  `TOTP_ENCRYPTION_KEY`, so the API starts from a copied `.env`.

- The templates move to rok-data 0.9. `rok-data.pin` is v0.9.0; the
  templates' rok-data dependency is `"0.9"` (manifests and the READMEs
  quoting them), their lockfiles take the published 0.9.0 runtime (exactly
  rok-data, rok-data-core and rok-data-macros move), and their workflows'
  `ROK_VERSION` is 0.9.0. Regenerated by the 0.9.0 generator built against
  this checkout: each `db/mod.rs`'s dependency hint and `db/schema.rs`'s
  `SCHEMA_SQL` doc change; no query's type does, and every template query
  types under the now-strict `rok gen` as it is. The monorepo's Clients
  gain header parameters (`If-Match` and `If-None-Match` values quoted as
  entity tags) and the descriptions of nullable enum properties; no table
  of the template is versioned, so no call changes and the web, mobile
  and desktop Apps are unchanged.
- The API templates' READMEs name `[api] include`, the allowlist
  alternative to `[api] exclude`, and `rok gen`'s notice of a newly served
  table.

**Fixed**

- The API templates' database tests can't pass unrun in CI: with `CI` set
  and no `DATABASE_URL` (exported or in `.env`), `Database::fresh` fails
  with "DATABASE_URL is required in CI"; locally they still skip with a
  note (`tests/support`'s `database_url`, in the actix, axum, gRPC and
  monorepo API templates).
- Monorepo web App: its tests pass on Node 25 and later. Node 25 defines a
  `localStorage` global, `undefined` without `--localstorage-file`, that
  hid jsdom's, so every test using storage failed; `vitest.config.ts`
  now runs the tests with `--no-experimental-webstorage`. CI also tests
  the web App on the current Node release, beside the pinned Node 22.
- actix: `AuthUser` and `AdminUser` check the caller before the body is
  read, as `Allowed<G>` does, so an unauthenticated call with a bad body
  answers `401` and a non-admin `403`, not the body's `422`. actix-web
  polls a handler's extractors together; each caller extractor now takes
  a hold on the body (`shared/body.rs`'s `hold`, opened once the caller
  is let in, the body read as empty if refused), and the `hold_body`
  middleware `app` wraps every request in gates the body up front, so the
  hold works whichever order a handler takes the caller and the body in,
  generated handlers included. `Allowed<G>` moves onto the same hold
  (its body hold now works in any order too). A wrong `Content-Type` or
  an over-limit `Content-Length` is still answered first. axum is
  unchanged: its extractors already run in order.

## [0.7.0]

The `grpc` template, the templates on rok-data 0.8.1, and their CI
installing the `rok` release.

**Added**

- The `grpc` template (`templates/grpc-api/`, workspace ADR 0030): the axum
  template's schema generated with `framework = "grpc"` (engine ADR 0028),
  a `.proto` per Resource under `proto/` and a tonic service per feature,
  compiled by `build.rs` with protox, so no `protoc` is needed; the tonic
  server, with `grpc.health.v1.Health` (`NOT_SERVING` while draining) and
  server reflection outside the caller check; the axum template's
  configuration, data layer sources, jobs, mail, storage, telemetry,
  metrics, shutdown and security, which `template_drift` holds identical;
  and tests that call the services in process (CRUD against PostgreSQL
  when `DATABASE_URL` is set). The gRPC stack is pinned exactly (tonic,
  tonic-health, tonic-reflection, tonic-prost, tonic-types and
  tonic-prost-build 0.14.9, prost 0.14.4, protox 0.10.0).
- The `grpc` template's caller check (`app/auth.rs`, a tower layer in
  front of the API's services): it accepts the axum template's bearer
  tokens (the same `JWT_SECRET`, `JWT_SECRET_PREVIOUS` and claims; the
  account's `token_version`, activity and role from the database) and its
  API keys (`authorization: Bearer rok_…` or `x-api-key`; a `read` key
  calls only the RPCs named in `READS`, `Get` and `List`), and puts the
  `AuthUser` in the request's extensions. Invalid credentials are
  `UNAUTHENTICATED` (a `read` key on a write `PERMISSION_DENIED`); a call
  with none reaches the services without a caller, so the generated ones
  answer `UNAUTHENTICATED`. Health and reflection stay outside it. `tower`
  0.5.3 becomes a dependency, pinned exactly (the lockfile already carried
  it). `tests/auth.rs` calls through it with real tokens and API keys.
- The `grpc` template's streaming example: `UserEventsService.Subscribe`, a
  hand-written server-streaming RPC (`proto/user_events.proto`, importing
  the generated `User`) in a hand-written `user_events` feature behind the
  caller check, admins only; it is a read (`READS`), so an admin's `read`
  API key subscribes. Each `NOTIFY user_events, '<user id>'` is one
  event (the user as it is now, or the id alone once it is gone): the
  service publishes with `pg_notify`, and any process sharing the database
  may too; no migration is added. Each subscriber listens on a connection
  of its own (`PgListener`) through a bounded queue: hanging up drops the
  listener, falling 16 events behind ends the stream with
  `RESOURCE_EXHAUSTED`, a lost connection with `UNAVAILABLE`. tokio-stream
  (=0.1.19) moves from the dev-dependencies to the dependencies.
- A `grpc_template` gate: drift against generation, rustfmt, clippy and the
  template's tests, with `PROTOC` naming no program.

**Fixed**

- The templates' CI (actix-api, axum-api and the monorepo's API App)
  installs `rok` from its release: it downloads the
  `x86_64-unknown-linux-musl` archive of the rok release the workflow's
  `ROK_VERSION` pins (0.8.0) from rok-v2's GitHub releases, checks it
  against its `.sha256` and puts `rok` on the PATH. It ran `cargo install
  rok-data-cli --locked`, which built from source on every run and
  installs no `rok`, so the generated-code and architecture checks failed.
  `tests/ci_rok_version.rs` checks every template workflow pins the same
  rok release, of the rok-data minor in `rok-data.pin`.

**Changed**

- `rok-data.pin` is `v0.8.1`, the release whose gRPC services compile with
  an `[api.immutable]` column. The templates' lockfiles move to the
  published 0.8.1 runtime (exactly rok-data, rok-data-core and
  rok-data-macros move); their manifests keep `"0.8"`. The `grpc` template
  takes back the axum template's `[api.immutable] users = ["email"]`: its
  `UpdateUser` message no longer has an `email`, an update naming one is
  refused, and its generated `users/service_api.rs` is the axum template's
  again, which `template_drift` now holds identical. The note on the
  workaround leaves its README and `rok-data-sqlc.toml`.
- `rok-data.pin` is `v0.8.0` (the release carrying `framework = "grpc"`),
  and the templates move to rok-data 0.8: their manifests, the READMEs
  that quote them, their lockfiles (exactly rok-data, rok-data-core and
  rok-data-macros move) and their generated files (only each
  `db/mod.rs`'s dependency hint).
- `rok-data.pin` is `v0.7.0`, and the templates move to rok-data 0.7: their
  manifests, the READMEs that quote them, their lockfiles (exactly
  rok-data, rok-data-core and rok-data-macros move) and their generated
  files (only each `db/mod.rs`'s dependency hint).
- The gates also refuse a generator whose embedded axum `Cargo.lock`
  differs from this checkout's: one built before a lockfile change made
  the `--locked` builds fail far from the cause.

**Changed**

- `rok-data.pin` is `v0.6.1`: the first 0.6 release of rok-data whose crates
  all published (0.6.0 stopped before `rok-data-sqlc`).

## [0.6.0]

The first release from this repository: everything that landed in
rok-data's `crates/rok-templates` after 0.5.0.

**Added**

- The `monorepo` template (rok-data's workspace ADR 0027): an axum API
  with web (TanStack Start), mobile (Flutter) and desktop (GPUI, ADR
  0031) Apps and the generated `clients/ts`, `clients/dart` and
  `clients/rust`.
- The catalog: `catalog::files`, `catalog::apps`, `names` and `rename`,
  which map a template name to the files a project gets and rename its
  Apps after the project at init.

**Changed**

- The crate has its own repository (rok-data's workspace ADR 0032),
  history included; it was `crates/rok-templates` in rok-data, where
  versions up to 0.5.0 are recorded in rok-data's `CHANGELOG.md`.
- The template gates moved here from rok-data's `fixture_compile` suite
  (`tests/gates/`). They generate with the rok-data release in
  `rok-data.pin`, built against this checkout.
