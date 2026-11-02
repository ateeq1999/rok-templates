# Changelog

All notable changes to `rok-templates` are documented here. The format
is [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and the
version numbers follow [Semantic
Versioning](https://semver.org/spec/v2.0.0.html). Release notes for a tag
are this file's `## [X.Y.Z]` section.

## [Unreleased]

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
