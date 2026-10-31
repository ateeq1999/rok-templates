# Getting Started

This guide gets the starter running on a device or emulator in ~10 minutes. It assumes a working Flutter 3.11+ SDK (`flutter doctor` green).

## 1. Clone & rename

```bash
git clone <your-fork-url> myapp
cd myapp
```

Then change these identifiers to your own:

- `name:` in [pubspec.yaml](../pubspec.yaml) (currently `mobile`)
- Application ID / Bundle ID: run `flutter create --org com.yourcompany --platforms ios,android .` in-place to re-seed the native folders, **or** edit `android/app/build.gradle` (`applicationId`) and `ios/Runner.xcodeproj` (`PRODUCT_BUNDLE_IDENTIFIER`) manually.
- Deep link scheme: see step 5.

## 2. Install dependencies

```bash
flutter pub get
```

Generated files (`*.freezed.dart`, `*.g.dart`) are committed. Re-run codegen only after editing a model:

```bash
dart run build_runner build --delete-conflicting-outputs
```

## 3. Configure `.env`

Copy the template:

```bash
cp .env.example .env
```

Edit [`.env`](../.env.example):

```env
API_URL=http://localhost:3000     # iOS sim + macOS/web
# API_URL=http://10.0.2.2:3000    # Android emulator (host machine loopback)

APP_SCHEME=myapp                  # deep link scheme (see step 5)

FEATURE_MAGIC_LINK=true
FEATURE_TWO_FACTOR=true
FEATURE_ORGANIZATIONS=true
FEATURE_ADMIN=true
FEATURE_SIGN_UP=true
FEATURE_OAUTH=false
FEATURE_PUSH_NOTIFICATIONS=true
FEATURE_THEME_CUSTOMIZATION=true
FEATURE_DELETE_ACCOUNT=false
```

`.env` is listed in [`pubspec.yaml` assets](../pubspec.yaml#L80-L82) and loaded at startup by [main.dart:37](../lib/main.dart#L37). See [feature-flags.md](./feature-flags.md) for flag semantics.

## 4. Firebase setup

Push notifications and FCM require a Firebase project.

```bash
dart pub global activate flutterfire_cli
flutterfire configure
```

This writes `lib/firebase_options.dart` and the platform config files. If you don't want push notifications, set `FEATURE_PUSH_NOTIFICATIONS=false` — but Firebase still initialises on startup, so you'll need a valid `firebase_options.dart` either way (a minimal no-op project works).

Required files:

- `android/app/google-services.json`
- `ios/Runner/GoogleService-Info.plist`

## 5. Deep links

Default scheme is `myapp://`. Change it in three places:

1. `.env` → `APP_SCHEME=myapp`
2. [android/app/src/main/AndroidManifest.xml](../android/app/src/main/AndroidManifest.xml) — `<data android:scheme="myapp"/>`
3. `ios/Runner/Info.plist` → `CFBundleURLSchemes`

Supported inbound links (handled in [main.dart:138-175](../lib/main.dart#L138-L175)):

| URL | Effect |
|---|---|
| `myapp://auth/callback?token=...` | OAuth token → save → `/home` |
| `myapp://auth/verify-email?token=...` | → `/verify-email` |
| `myapp://auth/reset-password?token=...` | → `/reset-password` |
| `myapp://auth/magic-link?token=...` | → `/magic-link` (auto-verifies) |
| `myapp://invite?token=...` | → `/invitations/accept` |

## 6. Run

```bash
flutter run
```

Splash → redirects based on [AuthBloc](../lib/app/modules/auth/auth_bloc.dart) state:
- No stored token → `/sign-in`
- Valid token + user → `/home`

## 7. Verify

```bash
flutter test           # all unit + widget tests (should pass green)
flutter analyze        # lint + type check
```

## Troubleshooting

| Symptom | Fix |
|---|---|
| `Unable to load asset: .env` | `.env` missing — copy from `.env.example` |
| Android sign-in hangs | Using `localhost` instead of `10.0.2.2` from emulator |
| iOS sim can't reach API | Use `localhost` (iOS sim shares host network) |
| `FirebaseException: [core/no-app]` | Run `flutterfire configure` — `firebase_options.dart` is out of sync |
| Deep link opens browser instead of app | Intent filter missing or `APP_SCHEME` mismatch; check step 5 |

Next: [architecture.md](./architecture.md) to understand how the layers fit together.
