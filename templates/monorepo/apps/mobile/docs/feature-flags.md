# Feature Flags

Every optional module in this starter is gated by a flag in `.env`. Flip a flag to `false` to hide the UI; delete the code to remove the feature entirely. This doc covers both.

## How flags work

Flags are loaded once at startup from `.env` via [FeatureFlags.fromEnv()](../lib/app/core/config/feature_flags.dart) and provided to the widget tree:

```dart
RepositoryProvider<FeatureFlags>.value(value: featureFlags)
```

Any widget reads them with:

```dart
final flags = context.read<FeatureFlags>();
if (flags.organizations) {
  // render the Organizations tile
}
```

Parsing rules ([feature_flags.dart:44-48](../lib/app/core/config/feature_flags.dart#L44-L48)):

- `false`, `0`, `no`, `off` → `false`
- anything else (or absent) → `true`
- **except** `FEATURE_OAUTH` and `FEATURE_DELETE_ACCOUNT` which default to `false`

## The nine flags

| Flag | Default | Gates |
|---|---|---|
| `FEATURE_MAGIC_LINK` | `true` | Magic-link button on [sign_in_view.dart](../lib/app/modules/auth/sign_in/sign_in_view.dart) + `/magic-link` route |
| `FEATURE_TWO_FACTOR` | `true` | 2FA tile in [settings_view.dart](../lib/app/modules/settings/settings_view.dart) + `/two-factor` route |
| `FEATURE_ORGANIZATIONS` | `true` | Organizations tile on [home_view.dart](../lib/app/modules/home/home_view.dart) + `/organizations*` routes |
| `FEATURE_ADMIN` | `true` | Admin tile on home (requires `user.role == 'admin'`) + `/admin*` routes |
| `FEATURE_SIGN_UP` | `true` | "Sign Up" link on sign-in view + `/sign-up` route |
| `FEATURE_OAUTH` | `false` | OAuth provider buttons (placeholder — wire your own providers) |
| `FEATURE_PUSH_NOTIFICATIONS` | `true` | FCM init + device token registration |
| `FEATURE_THEME_CUSTOMIZATION` | `true` | Appearance link in settings + `/settings/appearance` route |
| `FEATURE_DELETE_ACCOUNT` | `false` | Delete-account tile in settings |

## Toggling a flag (keep the code)

Edit `.env`:

```diff
-FEATURE_ORGANIZATIONS=true
+FEATURE_ORGANIZATIONS=false
```

Hot-restart — UI gates flip immediately. Code stays on disk; the feature can be re-enabled per-build by shipping a different `.env`.

**Gotcha:** flags only hide widgets that actually check them. If you add a new screen, add the `flags.xxx` check yourself — see [home_view.dart:44-56](../lib/app/modules/home/home_view.dart#L44-L56) for the pattern.

## Dropping a feature (delete the code)

To remove a feature entirely (smaller bundle, fewer routes, no dead code):

### 1. Delete the module folder

```bash
rm -rf lib/app/modules/organizations
rm -rf lib/app/data/repositories/organization.repository.dart
rm -rf lib/app/data/providers/organization.provider.dart
rm -rf lib/app/data/models/organization.model.dart
rm -rf lib/app/data/models/org_member.model.dart
rm -rf lib/app/data/models/org_invitation.model.dart
```

### 2. Remove route constants

Delete the organizations section from [app_routes.dart](../lib/app/routes/app_routes.dart):

```dart
// Organizations
static const organizations = '/organizations';
static const orgInvitationAccept = '/invitations/accept';
static String orgDetail(String id) => '/organizations/$id';
static String orgInvite(String id) => '/organizations/$id/invite';
```

### 3. Remove routes + imports from the router

In [app_router.dart](../lib/app/core/router/app_router.dart), delete:

- The import lines for `organizations/*`
- The `OrganizationRepository` constructor parameter (if any)
- The four `GoRoute(path: AppRoutes.organizations…)` entries

### 4. Remove wiring from `main.dart`

In [main.dart](../lib/main.dart):

- Drop `OrganizationProvider` / `OrganizationRepository` construction (lines 57, 61)
- Drop the `orgRepository` parameter + `RepositoryProvider<OrganizationRepository>`
- Drop the `invite` / `invitations` deep link branch in `_handleDeepLink`

### 5. Remove the `organizations` field from `FeatureFlags`

In [feature_flags.dart](../lib/app/core/config/feature_flags.dart), delete the field, the constructor parameter, and the `fromEnv` line. Remove `FEATURE_ORGANIZATIONS` from `.env` and `.env.example`.

### 6. Remove UI references

Grep for residuals and delete them:

```bash
grep -r "organizations" lib/
grep -r "Organization" lib/
```

Common spots: [home_view.dart](../lib/app/modules/home/home_view.dart) (tile), deep link handler in [main.dart](../lib/main.dart).

### 7. Delete the tests

```bash
rm test/unit/organization_repository_test.dart
```

### 8. Verify

```bash
flutter analyze
flutter test
```

Both should be green. If `analyze` complains about missing imports, keep deleting until it's quiet.

## Same steps apply to every optional module

| To drop | Module folder | Repository / provider / models | Flag field |
|---|---|---|---|
| Magic link | `modules/auth/magic_link/` | `auth.provider.dart::sendMagicLink/verifyMagicLink` + `auth.repository.dart` counterparts | `magicLink` |
| 2FA | `modules/auth/two_factor/` | `auth.provider.dart::enableTwoFactor/verifyTotp/disableTwoFactor` | `twoFactor` |
| Admin | `modules/admin/` | `admin.{provider,repository}.dart` + models `admin_user`, `admin_stats`, `audit_log` | `admin` |
| Sign up | `modules/auth/sign_up/` | `auth.provider.dart::signUp` | `signUp` |
| Theme customization | `modules/theme/` | `theme_cubit.dart`, `core/theme/app_theme.dart` prefs logic | `themeCustomization` |
| Delete account | just the settings tile | `user.{provider,repository}.dart::deleteAccount` | `deleteAccount` |
| Push notifications | `services/notification_service.dart` | `user.{provider,repository}.dart::registerDeviceToken/deleteDeviceToken` + Firebase packages | `notifications` |

## Adding a new flag

1. Add the field + constructor + `fromEnv` entry in [feature_flags.dart](../lib/app/core/config/feature_flags.dart).
2. Add `FEATURE_XXX=true` to both `.env` and [.env.example](../.env.example).
3. Read it where it gates UI: `context.read<FeatureFlags>().xxx`.
4. Update the table in this doc.

## Runtime vs. compile-time

These flags are **runtime** (read from `.env` at startup). They don't shrink the compiled binary — dead-code elimination can't tell that a flag will always be `false`. If binary size matters, delete the code (above) rather than toggling.

For true compile-time flags, use `--dart-define=FEATURE_X=true` and read via `const bool.fromEnvironment('FEATURE_X')` — those get constant-folded and the dead branch is stripped.
