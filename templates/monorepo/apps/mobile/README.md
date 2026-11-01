# The mobile App

The monorepo's Flutter app (iOS, Android, and the desktop and web targets
Flutter builds). It signs in against `apps/api` and calls it only through
the generated Dart client in `../../clients/dart` (`package:api_client`),
which `rok gen` writes from the API's `openapi.json`: the HTTP calls, the
models and the error type are generated, never hand-written here.

State management is **`flutter_bloc`**; navigation is **`go_router`**.

Its home is rok-data's `rok-templates` crate
([`crates/rok-templates/templates/monorepo/apps/mobile`](https://github.com/ateeq1999/rok-data/tree/main/crates/rok-templates/templates/monorepo/apps/mobile)):
the `Nestjs-Better-Auth-Flutter-Starter` repository it was imported from
is no longer maintained. [The monorepo
guide](https://github.com/ateeq1999/rok-data/blob/main/crates/rok-templates/docs/guide/monorepo.md)
describes the project around it.

## Run it against the local API

```bash
# 1. the API (from the monorepo's root; see apps/api/README.md)
cargo run -p axum-api

# 2. the App
cd apps/mobile
flutter pub get
cp .env.example .env          # optional: API_URL, APP_SCHEME, feature flags
flutter run --dart-define-from-file=.env
```

Without `API_URL` the App calls `http://127.0.0.1:8080` (apps/api's
`BIND_ADDR`), or `http://10.0.2.2:8080` from an Android emulator, which
is the host machine's `127.0.0.1`. A phone on the network needs the
machine's address in `API_URL` and the API bound to it.

Checks:

```bash
flutter analyze
flutter test
```

## What's in the box

| Module | Flag | Calls |
|---|---|---|
| Sign in, with the two-factor step | always on | `auth.login`, `auth.verify2fa`, `auth.me` |
| Sign up | `FEATURE_SIGN_UP` | `auth.register` (email and password; the API mails a verification link) |
| Forgot / reset password | always on | `auth.forgot`, `auth.reset` |
| Email verification | always on | `auth.verifyEmail`; a new link from the profile (`auth.resend`) |
| Profile: display name, photo | always on | `users.me`, `users.updateMe`, `avatars.upload` |
| Change password | always on | `auth.change` (the API ends every session; sign in again) |
| Two-factor (TOTP) | `FEATURE_TWO_FACTOR` | `auth.setup`, `auth.enable` (recovery codes), `auth.disable` |
| Theme customization | `FEATURE_THEME_CUSTOMIZATION` | none (on the device) |

Flags are build-time `--dart-define`s ([docs/feature-flags.md](docs/feature-flags.md)).

**Not in the App until `apps/api` serves them** — the Flutter starter this
App was imported from had them against another backend; they are removed,
not left half-working: organizations, magic-link sign-in, the admin panel,
deleting the account, OAuth sign-in, push notifications (a recipe:
[docs/push-notifications.md](docs/push-notifications.md)), and a sign-out
call (the API has none: signing out forgets the token on the device).

## Talking to the API

```dart
final api = ApiClient(baseUrl: AppConfig.apiUrl, session: authService);
final user = await api.users.me();
```

`AuthService` (`lib/app/services/auth_service.dart`) is the client's
`Session`: it keeps the bearer token in `flutter_secure_storage`, the client
sends it on every secured call, and a `401` to it signs out. Every failure
is the client's `ApiException` (`status`, `code`, `message`, and a `422`'s
field issues, which forms show through `fieldErrors`).

Change the API, then run `rok gen` from the monorepo's root: it re-exports
`apps/api/openapi.json` and regenerates the client. Never edit
`clients/dart` by hand.

## Layers

```
View          widgets, side effects (snackbar, navigation)
  │ context.read<Cubit>()
Cubit / BLoC  sealed UI states (*Initial, *Loading, *Loaded, *Failure)
  │
Repository    the App's use of the API: `api.<tag>.<operation>(...)`
  │
ApiClient     clients/dart, generated (bearer token from AuthService)
```

More: [architecture](docs/architecture.md), [adding a feature](docs/adding-a-feature.md),
[security](docs/security.md), [getting started](docs/getting-started.md).

## Firebase

Firebase configuration names one Firebase project, so the App ships
placeholders (`android/app/google-services.json`,
`ios/Runner/GoogleService-Info.plist`, project `your-firebase-project`)
and does not start Firebase. [docs/push-notifications.md](docs/push-notifications.md)
points to the recipe that makes it real.
