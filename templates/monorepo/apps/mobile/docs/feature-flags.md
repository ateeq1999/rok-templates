# Feature Flags

Every optional module the App ships is gated by a build-time flag. Turn
a flag off to hide the UI; delete the code to remove the feature entirely.

## How flags work

Flags are `--dart-define`s, read by
[FeatureFlags.fromEnvironment()](../lib/app/core/config/feature_flags.dart)
and provided to the widget tree:

```dart
RepositoryProvider<FeatureFlags>.value(value: featureFlags)
```

Pass them one by one, or from the `.env` file (copy `.env.example`):

```bash
flutter run --dart-define-from-file=.env
flutter run --dart-define=FEATURE_SIGN_UP=false
```

Any widget reads them with:

```dart
final flags = context.read<FeatureFlags>();
if (flags.twoFactor) {
  // render the two-factor tile
}
```

Parsing: `false`, `0`, `no`, `off` → `false`; anything else (or absent) → `true`.

## The flags

| Flag | Default | Gates |
|---|---|---|
| `FEATURE_TWO_FACTOR` | `true` | 2FA tile in [settings_view.dart](../lib/app/modules/settings/settings_view.dart) |
| `FEATURE_SIGN_UP` | `true` | "Sign Up" link on the sign-in view |
| `FEATURE_THEME_CUSTOMIZATION` | `true` | Appearance link in settings + `/settings/appearance` route |

**Gotcha:** flags only hide widgets that check them. A new screen adds its
own `flags.xxx` check.

## What is not here, and why

The Flutter starter this App was imported from also had organizations,
magic-link sign-in, an admin panel, deleting the account, OAuth sign-in
and push notifications, against a NestJS / Better-Auth backend. `apps/api`
does not serve them, so they are not in the App, rather than behind a flag
that would show screens with nothing to call:

| Feature | What `apps/api` lacks |
|---|---|
| Organizations (list, detail, members, invitations) | any organization endpoint |
| Magic-link sign-in | a magic-link endpoint |
| Admin panel (users, audit log, stats) | the stats and per-user actions the panel used; `users.list` and `auditLog.list` exist, so a reduced panel can come back through the client |
| Delete account | a self-service delete (`DELETE /users/{id}` is an admin's) |
| OAuth sign-in | OAuth endpoints |
| Push notifications | a device-token endpoint ([recipe](./push-notifications.md)) |
| Sign out on the server | a sign-out endpoint: signing out forgets the token on the device |

When the API grows one, `rok gen` adds its operations to `clients/dart`;
add a repository method, a cubit and a view ([adding a feature](./adding-a-feature.md)),
and a flag if it should be optional.

## Dropping a feature (delete the code)

To remove a module entirely (smaller bundle, fewer routes):

1. Delete its folder under `lib/app/modules/` and its repository methods.
2. Remove its routes from [app_router.dart](../lib/app/core/router/app_router.dart)
   and its constants from [app_routes.dart](../lib/app/routes/app_routes.dart).
3. Remove its flag (field, `fromEnvironment` entry, `.env.example` line).
4. Delete its tests, then `flutter analyze` and `flutter test` until both
   are green.

## Adding a new flag

1. Add the field, its `String.fromEnvironment` constant and the
   `fromEnvironment` entry in [feature_flags.dart](../lib/app/core/config/feature_flags.dart).
2. Add `FEATURE_XXX=true` to [.env.example](../.env.example).
3. Read it where it gates UI: `context.read<FeatureFlags>().xxx`.
4. Update the table in this doc.

## Compile-time

The flags are compiled in (`String.fromEnvironment` is a constant), so a
build carries the values it was made with; changing one means building
again, not restarting.
