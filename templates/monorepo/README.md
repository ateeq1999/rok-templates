# The monorepo starter

One backend, a web app, a desktop app and a mobile app sharing one API
contract (workspace ADRs 0027 and 0031). The Apps `rok sqlc init
--template monorepo` scaffolds:

- `apps/api/` — the axum API, the `axum` template file for file. Its own
  README is its reference; see it there.
- `apps/web/` — the web app: TanStack Start (React, TypeScript), run with
  plain bun (no JS monorepo tool). It talks to `apps/api` through the
  generated `clients/ts`.
- `apps/desktop/` — the desktop app: rok-ui's bloc-http starter (GPUI), a
  member of the root workspace. It talks to `apps/api` through the
  generated `clients/rust`; its README is its reference.
- `apps/mobile/` — the mobile app: Flutter (`flutter_bloc`, `go_router`).
  It talks to `apps/api` through the generated `clients/dart`; what the API
  does not serve yet (organizations, magic link, admin, deleting the
  account, push notifications) is not in it. Its README is its reference.
- `clients/ts/` — the TypeScript Client for the API's contract, generated
  from `apps/api/openapi.json`. Its files are rok's: never edit them.
- `clients/dart/` — the Dart Client (the pub package `api_client`, for a
  Flutter App to depend on by path), generated from the same contract;
  rok's too.
- `clients/rust/` — the Rust Client (package `api-client`), generated from
  the same document: an async `reqwest` crate with its own serde types, a
  member of the root workspace. The api App's `tests/client/` drive the
  API through it. Its `src/` is rok's; its `Cargo.toml` is yours.

Start one, then generate the api's data layer and features (`src/db/`,
the generated feature files), which the scaffold leaves to `rok gen`:
`apps/api` does not build before it.

```sh
rok init --template monorepo my-app
cd my-app
rok gen
```

## Regenerating

`rok gen` from the root regenerates every Rust App the `[apps]` section
of `rok.toml` names, reading each App's settings from inside it; then it
re-exports the API's contract (`[contract]`, which builds `apps/api`) and
regenerates every Client `[clients]` names from it:

```sh
rok gen
```

`rok gen check` fails when a generated file, a Client included, is out
of date with its source; it builds nothing.

## The contract

`apps/api/openapi.json` is the OpenAPI document the API serves at
`/api-docs/openapi.json`, written out by its `openapi` example (what `rok
gen` runs):

```sh
cargo run -p axum-api --example openapi -- apps/api/openapi.json
```

The Clients are generated from it. Never edit it by hand: the api's
`tests/openapi_artifact.rs` fails when it is not what the API serves.

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
elsewhere (`ROK_API_URL=http://127.0.0.1:9000 cargo run -p desktop`).
`cargo build -p desktop --release` builds the binary alone, and `cargo
test -p desktop` is its check (no test touches the API or the keychain).
On Linux, GPUI needs its system libraries first (`apps/desktop/README.md`
lists them).

Sign up in the App, then verify the email: the API mails a link (with
its default `MAIL_TRANSPORT=log` it logs the mail instead; `RUST_LOG=debug`
shows the link), and the link's token goes to `POST
/api/v1/auth/email/verify` (the web App's `/verify-email` page sends it).
Then sign in. The App keeps the session's token in the OS keychain, one
entry per API address, so it starts signed in next time; signing out, or
the API refusing the token, deletes it. On Linux that is the kernel
keyring, which forgets the token when the machine restarts.

Signed in, every account reaches My account (its email, display name,
verified state and role). The users list and a user's detail are the
API's admin routes, so the App shows them only to an admin; anyone else
opening them is told they need the admin role. Make an account an admin in the database (`UPDATE users SET role = 'admin' WHERE
email = '...'`); the API reads the role on every request, so it applies
at once.

## Running the mobile app

You need Flutter. With the API running, from `apps/mobile`:

```sh
flutter pub get
flutter run
```

It calls the API at `http://127.0.0.1:8080` (`http://10.0.2.2:8080` from
an Android emulator); `--dart-define=API_URL=...`, or a `.env` passed with
`--dart-define-from-file=.env`, points it elsewhere (see
`apps/mobile/.env.example`). `flutter analyze` and `flutter test` are its
checks. Firebase ships as a placeholder: `apps/mobile/docs/push-notifications.md`
points to the recipe that makes it real.

## Where this comes from

The template lives in rok-data's `rok-templates` crate
(`crates/rok-templates/templates/monorepo/`), the only place its Apps are
maintained. [The monorepo
guide](https://github.com/ateeq1999/rok-data/blob/main/crates/rok-templates/docs/guide/monorepo.md)
walks the whole project: the Apps, the contract, the Clients, the `rok
gen` loop, and what the Apps leave out until the API serves it.
