# The desktop App

A [rok-ui](https://github.com/ateeq1999/rok-ui) (GPUI) desktop app, imported
from `cargo rok-ui new desktop --template bloc --http` at rok-ui 0.9.1
(workspace ADR 0031). Its layers are rok-ui's BLoC structure:

- `src/data/` — models, providers (the HTTP calls) and repositories.
- `src/features/` — one module per feature: its business logic (`bloc/`,
  free of GPUI) and its views (`view/`).
- `src/routes/` — thin route files; `build.rs` turns them into the
  `routes` module.
- `tests/features/` — each bloc's tests, against fake repositories.

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
