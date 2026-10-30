# The monorepo starter

One backend, a web app and a mobile app sharing one API contract
(workspace ADR 0027). The Apps `rok sqlc init --template monorepo`
scaffolds:

- `apps/api/` — the axum API, the `axum` template file for file. Its own
  README is its reference; see it there.
- `clients/` — the typed Clients for the API's contract, generated from
  the API's `openapi.json` (added as the template grows them).

Start one:

```sh
rok init --template monorepo my-app
cd my-app
```

## Regenerating the Apps

`rok gen` from the root regenerates every Rust App the `[apps]` section
of `rok.toml` names, reading each App's settings from inside it:

```sh
rok gen
```

## Running the API

You need PostgreSQL and Rust 1.94 or newer; `apps/api/README.md` has
each step. From the root:

```sh
cargo run -p axum-api
```
