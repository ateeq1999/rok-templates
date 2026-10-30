# The monorepo starter's agents

This is a Monorepo template scaffolded by `rok init --template
monorepo`: one backend, a web app, a desktop app and a mobile app sharing
one API contract (workspace ADRs 0027 and 0031).

- `apps/api/` — the axum API, the `axum` template file for file. Its
  `AGENTS.md` states the six architecture rules its code follows; its
  README is the reference for its settings and files.
- `apps/web/` — the web app (TanStack Start, plain bun); its checks are
  `bun install --frozen-lockfile`, `bun run typecheck`, `bun run test`.
- `apps/desktop/` — the desktop app (rok-ui on GPUI, package `desktop`, a
  root workspace member); run it with `cargo run -p desktop` while
  `apps/api` runs, check it with `cargo test -p desktop`. Its business
  logic (`src/features/<f>/bloc/`) stays free of GPUI (`clippy.toml`).
- `clients/` — generated Clients for the API's contract; do not edit a
  generated Client by hand.
- `rok.toml` names the Apps; `rok gen` from the root regenerates the
  generated code of every Rust App named there.
