# rok-templates

The project templates `rok init --template` (and the legacy `rok sqlc init
--template`) writes into a new project, and the catalog that names them:

| `--template` | What a project gets |
| --- | --- |
| `axum` | a complete axum API (`templates/axum-api/`): its schema, migrations, hand-written account flows and generated API features |
| `actix` | the same API on actix-web (`templates/actix-api/`) |
| `monorepo` | one backend and three apps on one API contract (`templates/monorepo/`): the axum API (`apps/api`), a TanStack Start web app (`apps/web`), a rok-ui desktop app (`apps/desktop`), a Flutter mobile app (`apps/mobile`), and the TypeScript, Dart and Rust clients generated from the API's OpenAPI document (`clients/`) |

You don't depend on this crate yourself. Install the CLI and start a
project from a template:

```console
$ cargo install rok-data-cli
$ rok init my-api --template axum
$ rok init my-shop --template monorepo --org com.acme
```

Each template's own `README.md` describes what it ships and which files
are yours. [The
guide](docs/guide/index.md)
explains the parts (the monorepo and its Apps, accounts, files, jobs,
observability, security, deploying to AWS) and has recipes for growing
them, push notifications for the mobile App among them. The actix and axum
templates share their framework-independent files byte for byte, and the
monorepo's `apps/api` is the axum template file for file;
`tests/template_drift.rs` checks both.

This crate is the only source of the monorepo's web and mobile Apps
(workspace ADR 0027): the starters they were imported from
(`ateeq1999/tanstack-starter-axum`,
`ateeq1999/Nestjs-Better-Auth-Flutter-Starter`) are not maintained any
more. Change the Apps here.

## The catalog

The crate is code as well as files (workspace ADR 0027, amending ADR
0022's data-only crate). `build.rs` embeds `templates/`, so `rok init`
works from `cargo install`, and
[`EMBEDDED`](https://docs.rs/rok-templates/latest/rok_templates/constant.EMBEDDED.html)
lists it. On top of it:

- `catalog`: the names `--template` takes, the template directory each
  resolves to, a Monorepo template's Apps, and the destination path each
  template file gets in a project;
- `names`: an App's names (packages, the Android `applicationId`, the iOS
  bundle id, the display name, the deep-link scheme) derived from the
  project's name and `--org`;
- `rename`: where in the template's files each of those names goes.

The write policy — copy when absent, skip files marked `@generated` — and
the grammar stay with the engine, `rok-data-sqlc`, whose `init` asks this
crate for the file plan and keeps the marker rules (workspace ADRs 0022
and 0027 in the repository's `docs/decisions/`).

Licensed under MIT OR Apache-2.0.
