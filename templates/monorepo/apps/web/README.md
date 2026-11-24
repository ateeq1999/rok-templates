# TanStack Start + shadcn/ui

This is a template for a new TanStack Start project with React, TypeScript, and shadcn/ui.

It is the web App of rok-data's monorepo template, and its home is the
`rok-templates` crate
([`crates/rok-templates/templates/monorepo/apps/web`](https://github.com/ateeq1999/rok-data/tree/main/crates/rok-templates/templates/monorepo/apps/web)
in rok-data): the `tanstack-starter-axum` repository it was imported from
is no longer maintained. [The monorepo
guide](https://github.com/ateeq1999/rok-data/blob/main/crates/rok-templates/docs/guide/monorepo.md)
describes the project around it.

## The dev server

`bun run dev` serves the App at `http://127.0.0.1:3001` and forwards `/api`
and `/health` to the API at `http://127.0.0.1:8080`. Both are IPv4
addresses, so scripts and `curl` use the same ones on every OS
(`localhost` is IPv6 alone on some systems, Windows among them).

To change them:

- **The dev server's address:** `server.host` in `vite.config.ts`, or
  `bun run dev --host 0.0.0.0` for one run (to open the App from another
  device on your network, for example). The port is the `--port 3001` of
  `package.json`'s `dev` script.
- **The API it forwards to:** the `API_PROXY_TARGET` environment variable,
  e.g. `API_PROXY_TARGET=http://127.0.0.1:9090 bun run dev` when the API's
  `BIND_ADDR` is on port 9090.

## Adding components

To add components to your app, run the following command:

```bash
npx shadcn@latest add button
```

This will place the ui components in the `components` directory.

## Using components

To use the components in your app, import them as follows:

```tsx
import { Button } from "@/components/ui/button";
```

## Talking to the API

The web App calls `apps/api` through the generated TypeScript client in
`../../clients/ts`, which `rok gen` writes from the API's `openapi.json`
(never edit it; change the API and run `rok gen` from the monorepo's root).
Import it from `@/lib/client`, which connects it to the session (the bearer
token, and signing out when the API rejects it):

```tsx
import { api, queries, mutations, ApiError } from "@/lib/client"

const me = useQuery(queries.users.me())
const update = useMutation(mutations.users.updateMe())
await api.auth.login({ email, password })
```

A feature's `queries.ts` adds what the App does around the calls
(invalidations, cache updates); its `schemas.ts` keeps the forms' rules and
names the client's types (`User` is the client's `UserResponse`).

Some of the starter's features call endpoints `apps/api` does not serve yet,
so their entry points are hidden until it does: passkeys, sign-in by QR
code, sign-in with Google or GitHub (and connected accounts), and creating a
user with a password (invite one instead). Their framework-free helpers
(`features/passkeys/webauthn.ts`, `features/oauth/messages.ts`) stay, with
their tests.
