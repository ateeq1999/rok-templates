# Getting Started

This gets the App running against the monorepo's API on a device or
emulator. It assumes a working Flutter SDK (`flutter doctor` green) and
`apps/api` set up as its README says.

## 1. Start the API

From the monorepo's root:

```bash
cargo run -p axum-api
```

It listens on `127.0.0.1:8080` (`BIND_ADDR` in `apps/api/.env`).

## 2. Install dependencies

```bash
cd apps/mobile
flutter pub get
```

`api_client` resolves by path from `../../clients/dart`, the client
`rok gen` generates from `apps/api/openapi.json`.

## 3. Configure (optional)

```bash
cp .env.example .env
```

| Setting | Default | Meaning |
|---|---|---|
| `API_URL` | `http://127.0.0.1:8080` (`http://10.0.2.2:8080` on an Android emulator) | The API's origin |
| `APP_SCHEME` | `myapp` | Deep-link scheme (step 5) |
| `FEATURE_*` | `true` | [Feature flags](./feature-flags.md) |

The values are compiled in: `flutter run --dart-define-from-file=.env`.
A physical phone needs the machine's network address in `API_URL`, and
the API bound to it.

## 4. Firebase

Not needed: the App does not start Firebase, and ships placeholders for its
configuration. [push-notifications.md](./push-notifications.md) adds it.

## 5. Deep links

`rok init` named the deep-link scheme after the project (its letters and
digits): `myapp://`. It is set in three places, so changing it later
changes all three:

1. `APP_SCHEME` (step 3)
2. [android/app/src/main/AndroidManifest.xml](../android/app/src/main/AndroidManifest.xml) — `<data android:scheme="myapp"/>`
3. `ios/Runner/Info.plist` → `CFBundleURLSchemes`

Inbound links (mapped to screens in
[deep_links.dart](../lib/app/core/router/deep_links.dart)):

| URL | Effect |
|---|---|
| `myapp://auth/verify-email?token=...` | → `/verify-email` |
| `myapp://auth/reset-password?token=...` | → `/reset-password` |
| `myapp://auth/oidc?code=...` (or `requires_totp`, `linked`, `error`) | → `/auth/oidc`, which finishes a sign-in with a provider |

`apps/api` mails links to `{APP_URL}/verify-email?token=...` and
`{APP_URL}/reset-password?token=...`; to have them open the App, route
them to the scheme above (an App Link / Universal Link on `APP_URL`, or a
redirect page there).

Sign-in with a provider (Google, say) opens the provider in the system
browser; the API's callback opens `myapp://auth/oidc` with a one-minute
code the App trades for a session. Set the API's `APP_SCHEME` to the same
scheme: without it, the API refuses a mobile sign-in. The App shows a
"Continue with …" button for each provider the API is configured with,
and none while it has none.

## 6. Run

```bash
flutter run --dart-define-from-file=.env
```

Splash → redirects based on [AuthBloc](../lib/app/modules/auth/auth_bloc.dart) state:
- No stored token → `/sign-in`
- Stored token + user → `/home`

Sign up, follow the verification link the API mails (with
`MAIL_TRANSPORT=log`, `apps/api`'s default, it logs the mail), then sign
in.

## 7. Verify

```bash
flutter analyze
flutter test
```

## Troubleshooting

| Symptom | Fix |
|---|---|
| "Cannot reach the server" on an Android emulator | `API_URL` names `localhost`: the emulator's own; leave it unset or use `10.0.2.2` |
| "Cannot reach the server" on a phone | `API_URL` must be the machine's network address, and the API bound to it |
| Signed out after changing the password | Expected: the API ends every session; sign in with the new password |
| Deep link opens browser instead of app | Intent filter missing or `APP_SCHEME` mismatch; check step 5 |

Next: [architecture.md](./architecture.md) to understand how the layers fit together.
