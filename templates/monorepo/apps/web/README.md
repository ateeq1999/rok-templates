# TanStack Start + shadcn/ui

This is a template for a new TanStack Start project with React, TypeScript, and shadcn/ui.

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
