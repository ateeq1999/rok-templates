# The monorepo template

`rok init --template monorepo` makes one backend and three apps that share
one API contract (workspace ADRs 0027 and 0031): the axum API, a web app,
a desktop app and a mobile app. The apps never hand-write a call to the
API: they call it through **Clients** rok generates from the API's
OpenAPI document, so a change to the API reaches every app through one
command, `rok gen`.

- [Starting one](#starting-one)
- [The Apps](#the-apps)
- [One contract](#one-contract)
- [The Clients](#the-clients)
- [The `rok gen` loop](#the-rok-gen-loop)
- [What is hidden, and why](#what-is-hidden-and-why)
- [Names](#names)

## Starting one

```console
$ rok init my-shop --template monorepo --org com.acme
$ cd my-shop
$ rok gen
```

`rok init` writes the project (every file once, never rewritten; a second
run fills in only what is missing) and prints the next steps. The api
App's generated code is not copied: the first `rok gen` writes it, then
re-exports the contract and regenerates the Clients from it (unchanged on
a fresh project: `rok init` wrote them from the same document). The
project:

```text
my-shop/
  Cargo.toml       the root workspace: apps/api, apps/desktop, clients/rust
  Cargo.lock
  rok.toml         the Apps rok gen regenerates, the contract, the Clients
  AGENTS.md
  README.md        how to run and check each App
  apps/
    api/           the axum API (package my-shop-api)
    web/           TanStack Start, run with plain bun (package my-shop-web)
    desktop/       rok-ui on GPUI (package my-shop-desktop)
    mobile/        Flutter (pub package my_shop_mobile)
  clients/
    ts/            the TypeScript Client
    dart/          the Dart Client (pub package api_client)
    rust/          the Rust Client (package api-client)
```

There is no JavaScript monorepo tool: `apps/web` is a plain bun project,
and the root `Cargo.toml` holds only the Rust members.

## The Apps

| App | What it is | Run it | Its checks |
| --- | --- | --- | --- |
| `apps/api` | The `axum` template file for file (`tests/template_drift.rs` checks it), plus the contract's export (`examples/openapi.rs`) and the tests that drive it through `clients/rust` (`tests/client/`). Every other page of this guide is about it. | `cargo run -p my-shop-api` from the root | `cargo test -p my-shop-api` (with `DATABASE_URL`; without it the database tests print a note and pass) |
| `apps/web` | TanStack Start (React, TypeScript, shadcn/ui), on `clients/ts`. | `bun install --frozen-lockfile`, then `bun run dev` in `apps/web` | `bun run typecheck`, `bun run test` |
| `apps/desktop` | rok-ui's bloc-http starter on GPUI, imported at rok-ui 0.9.1. | `cargo run -p my-shop-desktop` from the root | `cargo test -p my-shop-desktop` |
| `apps/mobile` | Flutter (`flutter_bloc`, `go_router`) for iOS and Android, on `clients/dart`. | `flutter pub get`, then `flutter run` in `apps/mobile` | `flutter analyze`, `flutter test` |

The API listens on `127.0.0.1:8080` (its `BIND_ADDR`), and every App calls
it there by default:

- the web dev server (port 3001) forwards `/api` and `/health` to it;
  `API_PROXY_TARGET` points it elsewhere (`apps/web/.env.example`);
- the desktop App reads `ROK_API_URL`;
- the mobile App reads `API_URL` from a `--dart-define`, or from a `.env`
  passed with `--dart-define-from-file=.env` (`apps/mobile/.env.example`);
  from an Android emulator its default is `http://10.0.2.2:8080`, the host.

Each App's own `README.md` is its reference; the project's `README.md`
lists them all.

## One contract

The **contract** is `apps/api/openapi.json`: the OpenAPI 3.1 document the
API serves at `/api-docs/openapi.json`, written to a file by the api's
`examples/openapi.rs`. `rok.toml` says where it is and how to write it:

<!-- source: templates/monorepo/rok.toml -->
```toml
[contract]
document = "apps/api/openapi.json"
export = ["cargo", "run", "--quiet", "-p", "axum-api", "--example", "openapi", "--"]
```

(`rok init` puts the api's package name, `my-shop-api`, in `export`.)
Never edit the document by hand: the api's `tests/openapi_artifact.rs`
fails when it is not what the API serves. The backend is the axum API
only; the actix template stays a single-app template, so there is one
contract, not two to keep apart.

## The Clients

<!-- source: templates/monorepo/rok.toml -->
```toml
[clients]
ts = "clients/ts"
dart = "clients/dart"
rust = "clients/rust"
```

rok emits each Client itself, from the contract, not from the engine's
model: the hand-written routes (sign-in, two-factor, files) exist only in
the document. All three name an operation the same way (`api.auth.verify2fa`
in TypeScript and Dart, `client.auth().verify2fa(...)` in Rust, whose
names are `snake_case`, such as `verify_email`), send the bearer token
only to the operations that need it, and report a failure as one error
type carrying the API's `422` field issues.

- **`clients/ts`**: a typed call per operation (`api.<tag>.<operation>`),
  TanStack Query `queryOptions` and `mutationOptions` factories
  (`queries`, `mutations`, `keys`), `ApiError`, and `configure({ baseUrl,
  token, onUnauthorized })`. `apps/web` imports it as `@client` and wires
  it to its session in `src/lib/client.ts`.
- **`clients/dart`**: the pub package `api_client` (it depends on `http`
  only): `ApiClient(baseUrl:, session:)`, a class per schema with
  `fromJson`/`toJson`, a `sealed class` per `oneOf`, `ApiException`.
  `apps/mobile` depends on it by path and implements its `Session`.
- **`clients/rust`**: the crate `api-client`, async on `reqwest`, with its
  own serde types, so a caller does not compile the server:
  `Client::new(base_url)`, `client.<tag>().<operation>(...)`, `set_token`,
  `on_unauthorized`, `keep_session()`, `ApiError`. The api's
  `tests/client/` start the API on a socket and drive it through this
  crate, so the server and the Client are proved to agree.

A Client's files carry the generated marker and are rok's: never edit
them. The one exception is `clients/rust/Cargo.toml`, written once by
`rok init` and yours to tune.

## The `rok gen` loop

Change the API (its schema, a hand-written route), then from the root:

```console
$ rok gen
```

It regenerates every App `[apps]` names (today the api: `src/db/`,
`src/shared/`, `src/features/`), re-exports the contract by running
`export` (which builds the api), and emits each Client from it: changed
files are rewritten, a file whose marker is gone is refused (the run fails)
unless `--force`, a marked file no longer emitted is removed. Each App then
meets the change where its compiler can see it: `bun run typecheck` in
`apps/web`, `flutter analyze` in `apps/mobile`, `cargo test` for the api
(whose `tests/client/` call it through `clients/rust`).

A schema change is the same loop with a migration. Add a `bio TEXT`
column to `users` in `apps/api/schema/`, then:

```console
$ rok gen --migration add_bio
```

It stages `apps/api/migrations/0009_add_bio.up.sql` (`ALTER TABLE "users"
ADD COLUMN "bio" TEXT`) and its `.down.sql`, regenerates the api's data
layer and features, re-exports the contract, and the three Clients gain
the field (`UpdateUserDto.bio` in TypeScript, for one). The hand-written
code that builds a `NewUser` (the api's test helpers) now fails to compile
until it says what `bio` is: the compiler finds every place the change
reaches. `rok migrate up --dir apps/api/migrations` applies the pair.

In CI:

```console
$ rok gen check
```

It builds and writes nothing: it fails when an App's generated code or a
Client is out of date with what it comes from. Whether the contract is
what the API serves is the api's own test. `rok gen --dry-run` neither
exports nor writes.

The web, desktop and mobile Apps have no generated code of their own, so
`[apps]` does not list them; they change only through their Clients.

## What is hidden, and why

The web and mobile Apps were imported from starters that called other
backends, and the desktop App from rok-ui's starter. A feature the API does
not serve is removed, not left half-working, and no API route was invented
for it. Each comes back by adding the endpoint to `apps/api`, running
`rok gen`, and calling it through the Client.

The web App (`apps/web`):

- passkeys, sign-in by QR code, and sign-in with Google or GitHub (OAuth):
  the API serves none of their endpoints;
- creating a user with a password from the admin pages: the API has no
  such endpoint (inviting a user stays).

The mobile App (`apps/mobile`, also listed in its `docs/feature-flags.md`):

- organizations, magic-link sign-in, OAuth sign-in: no endpoints;
- the admin panel: the statistics and per-user actions it used are not
  served (`users.list` and `auditLog.list` are, so a reduced panel can
  come back);
- deleting the account: only an admin may delete a user;
- server-side sign-out: the API has none; signing out forgets the token
  on the device;
- push notifications: they pick a vendor, so they are a recipe
  ([Push notifications](recipes/push-notifications.md)), not a default
  (workspace ADR 0023). The Firebase files ship as placeholders.

The desktop App (`apps/desktop`) is rok-ui's starter with its data layer
on `clients/rust`: its repositories call the generated client (sign-in,
the signed-in account, a page of users and one user), and a 422's broken
rules reach its form fields. The starter's notes feature is gone, as the
API serves no notes. Sign-up, the session kept in the OS keychain, and
the users screens are the next steps of workspace ADR 0031.

## Names

`rok init` names every App after the project's directory and `--org`
(default `com.example`): for `my-shop --org com.acme`, the packages
`my-shop-api`, `my-shop-web`, `my-shop-desktop` and `my_shop_mobile`, the
Android `applicationId` `com.acme.my_shop`, the iOS bundle id
`com.acme.myShop`, the display name `My Shop` and the deep-link scheme
`myshop`. The full table, and the names it refuses, are in the CLI
reference ([`rok init`](https://github.com/ateeq1999/rok-data/blob/main/docs/cli-reference.md#rok-init--scaffold-a-project)).
Run it once, at the start: renames follow the scaffolded-file rules, so a
file the project already holds keeps its names.
