# The monorepo starter's agents

This is a Monorepo template scaffolded by `rok init --template
monorepo`: one backend, a web app and a mobile app sharing one API
contract (workspace ADR 0027).

- `apps/api/` — the axum API, the `axum` template file for file. Its
  `AGENTS.md` states the six architecture rules its code follows; its
  README is the reference for its settings and files.
- `clients/` — generated Clients for the API's contract; do not edit a
  generated Client by hand.
- `rok.toml` names the Apps; `rok gen` from the root regenerates the
  generated code of every Rust App named there.
