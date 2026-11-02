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
  `apps/api` runs (`ROK_API_URL` when the API is not at
  `http://127.0.0.1:8080`), build it with `cargo build -p desktop`, check
  it with `cargo test -p desktop` and `cargo clippy -p desktop
  --all-targets`. On Linux, GPUI needs the system libraries its README
  lists. Its business logic (`src/features/<f>/bloc/`) stays free of GPUI
  (`clippy.toml`). It calls the API only through the generated
  `clients/rust` (`api_client`): never add HTTP code or JSON models by
  hand. The session's token lives in the OS keychain behind the
  `TokenStore` trait (`src/data/providers/token_store.rs`); tests use
  `MemoryTokenStore` and must never touch the real keychain.
- `apps/mobile/` — the mobile app (Flutter, package `mobile`); its checks
  are `flutter analyze` and `flutter test`. It calls the API only through
  the generated `clients/dart` (`package:api_client`): never add HTTP code
  or JSON models by hand; a missing call means a change to `apps/api` and
  `rok gen`.
- `clients/` — generated Clients for the API's contract; do not edit a
  generated Client by hand. `clients/rust` (package `api-client`, a root
  workspace member) is the Rust one: its `src/` is generated, its
  `Cargo.toml` is yours; check it with `cargo clippy -p api-client` and
  the api's `tests/client/`, which call the API through it.
- `rok.toml` names the Apps; `rok gen` from the root regenerates the
  generated code of every Rust App named there. `apps/api` does not
  build before `rok gen` has generated its data layer and features: run
  it first in a fresh checkout.
