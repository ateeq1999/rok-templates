# The desktop App

A [rok-ui](https://github.com/ateeq1999/rok-ui) (GPUI) desktop app, imported
from `cargo rok-ui new desktop --template bloc --http` at rok-ui 0.9.1
(workspace ADR 0031). Its layers are rok-ui's BLoC structure:

- `src/data/` — models, providers and repositories. The one provider,
  `providers/api.rs`, is the API's generated client (`clients/rust`, the
  `api_client` crate), wired to the app's session; the repositories call
  it (`api.client().users().list(&query)`) and turn its errors into the
  `ApiError` blocs and forms read, a 422's broken rules kept by field.
  `providers/token_store.rs` keeps the session's token in the OS keychain
  (`keyring`) between launches: signing in saves it, signing out or a 401
  deletes it. Tests keep it in memory (`MemoryTokenStore`) and never touch
  the keychain.
- `src/features/` — one module per feature: its business logic (`bloc/`,
  free of GPUI) and its views (`view/`).
  `features/users` is the users list (paged as the API pages it) and a
  user's detail, each read through its own bloc.
- `src/shell.rs` — the signed-in shell: rok-ui's `AdaptiveScaffold`
  (a bottom bar, a rail or a rail with labels, as wide as the window is),
  the Rok theme in the system's light or dark, and an app bar that toggles
  light and dark and signs out.
- `src/routes/` — thin route files; `build.rs` turns them into the
  `routes` module. The signed-in pages live under `_app/` (no path
  segment of its own), inside the shell: `/`, `/users`, `/users/:id`.
- `tests/data/` — the repositories' tests, the client answered by a mock
  API on localhost.
- `tests/features/` — each bloc's tests, against fake repositories; the
  auth flows (`auth/auth_flows_test.rs`) also through the real repository,
  the mock API and the in-memory token store.
- `tests/shell/` — the App itself rendered in GPUI's test context, in light
  and dark, its users screens answered by a mock API and its token kept in
  memory: a signed-out start lands on sign-in, and the shell's sign-out
  forgets the kept token.

Never add HTTP calls or JSON models by hand: a call the client lacks is a
change to `apps/api`, then `rok gen` from the root regenerates
`clients/rust`, and this App's build follows it (or breaks where it must
change).

Its home is rok-data's `rok-templates` crate
([`crates/rok-templates/templates/monorepo/apps/desktop`](https://github.com/ateeq1999/rok-data/tree/main/crates/rok-templates/templates/monorepo/apps/desktop));
[the monorepo
guide](https://github.com/ateeq1999/rok-data/blob/main/crates/rok-templates/docs/guide/monorepo.md)
describes the project around it.

## Running it

With `apps/api` running (see the monorepo's README), from the root:

```sh
cargo run -p desktop
```

The App calls the API at `http://127.0.0.1:8080`, `apps/api`'s `BIND_ADDR`.
Set `ROK_API_URL` to call it elsewhere:

```sh
ROK_API_URL=http://127.0.0.1:9000 cargo run -p desktop
```

## Checking it

```sh
cargo test -p desktop
cargo clippy -p desktop --all-targets
```

On Linux, GPUI needs its system libraries to build: on Debian or Ubuntu,
`sudo apt-get install libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev
libvulkan-dev libfontconfig-dev libx11-xcb-dev libxcb1-dev`.

`cargo rok-ui generate` (from rok-ui's `rok-ui-cli`) adds features and wires
them into `src/app.rs` at its `// rok-ui:` markers.
