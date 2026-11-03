# Changelog

All notable changes to `rok-templates` are documented here. The format
is [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and the
version numbers follow [Semantic
Versioning](https://semver.org/spec/v2.0.0.html). Release notes for a tag
are this file's `## [X.Y.Z]` section.

## [Unreleased]

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
- A `grpc_template` gate: drift against generation, rustfmt, clippy and the
  template's tests, with `PROTOC` naming no program.

**Changed**

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
