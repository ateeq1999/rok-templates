# rok-templates

The project templates `rok sqlc init --template` copies into a new
project: complete actix-web (`templates/actix-api/`) and axum
(`templates/axum-api/`) APIs, with their schema, migrations, hand-written
account flows and generated API features.

You don't depend on this crate yourself. Install the CLI and start a
project from a template:

```console
$ cargo install rok-data-cli
$ rok sqlc init --template axum
```

Each template's own `README.md` describes what it ships and which files
are yours. [The
guide](https://github.com/ateeq1999/rok-data/blob/main/crates/rok-templates/docs/guide/index.md)
explains the parts (accounts, files, jobs, observability, security,
deploying to AWS) and has recipes for growing them. The two templates
share their framework-independent files byte for byte;
`tests/template_drift.rs` checks that they stay so.

The crate is data only: `build.rs` embeds `templates/`, and
[`EMBEDDED`](https://docs.rs/rok-templates/latest/rok_templates/constant.EMBEDDED.html)
lists it. The engine, `rok-data-sqlc`, decides what a project gets
(workspace ADR 0022 in the repository's `docs/decisions/`).

Licensed under MIT OR Apache-2.0.
