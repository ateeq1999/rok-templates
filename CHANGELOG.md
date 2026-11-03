# Changelog

All notable changes to `rok-templates` are documented here. The format
is [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and the
version numbers follow [Semantic
Versioning](https://semver.org/spec/v2.0.0.html). Release notes for a tag
are this file's `## [X.Y.Z]` section.

## [Unreleased]

**Fixed**

- The templates' CI (actix-api, axum-api and the monorepo's API App)
  installs `rok` from its release: it downloads the
  `x86_64-unknown-linux-musl` archive of the rok release the workflow's
  `ROK_VERSION` pins (0.7.0) from rok-v2's GitHub releases, checks it
  against its `.sha256` and puts `rok` on the PATH. It ran `cargo install
  rok-data-cli --locked`, which built from source on every run and
  installs no `rok`, so the generated-code and architecture checks failed.
  `tests/ci_rok_version.rs` checks every template workflow pins the same
  rok release, of the rok-data minor in `rok-data.pin`.

**Changed**

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
