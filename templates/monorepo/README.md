# The monorepo starter

One backend, a web app, a desktop app and a mobile app sharing one API
contract (workspace ADRs 0027 and 0031). The Apps `rok sqlc init
--template monorepo` scaffolds:

- `apps/api/` — the axum API, the `axum` template file for file. Its own
  README is its reference; see it there.
- `apps/web/` — the web app: TanStack Start (React, TypeScript), run with
  plain bun (no JS monorepo tool). It talks to `apps/api` through its own
  HTTP layer today.
- `apps/desktop/` — the desktop app: rok-ui's bloc-http starter (GPUI), a
  member of the root workspace. It talks to `apps/api` through its own
  HTTP layer today; its README is its reference.
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

## Running the web app

You need bun. With the API running (it listens on `127.0.0.1:8080`, its
`BIND_ADDR`), from `apps/web`:

```sh
bun install --frozen-lockfile
bun run dev
```

The dev server (port 3001) forwards `/api` and `/health` to the API;
`API_PROXY_TARGET` points it elsewhere (see `apps/web/.env.example`).
`bun run typecheck` and `bun run test` are its checks.

## Running the desktop app

With the API running, from the root:

```sh
cargo run -p desktop
```

It calls the API at `http://127.0.0.1:8080`; `ROK_API_URL` points it
elsewhere. `cargo test -p desktop` is its check. On Linux, GPUI needs its
system libraries first (`apps/desktop/README.md` lists them).
