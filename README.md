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

The crate declares the templates' **catalog** (workspace ADR 0027): the
names `--template` takes, the template directories they resolve to, and
the destination path each template file gets in a project. `build.rs`
embeds `templates/`, [`EMBEDDED`](https://docs.rs/rok-templates/latest/rok_templates/constant.EMBEDDED.html)
lists it, and the catalog maps names onto it. The write policy — copy
when absent, skip files marked `@generated` — and the grammar stay with
the engine, `rok-data-sqlc`, which keeps the marker rules
(workspace ADRs 0022 and 0027 in the repository's `docs/decisions/`).

Licensed under MIT OR Apache-2.0.
