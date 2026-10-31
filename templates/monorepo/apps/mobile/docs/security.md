# Security

What this template does for you, what it doesn't, and what you need to decide before shipping.

## What's built in

### Token storage
Access + refresh tokens live in `flutter_secure_storage` — Keychain on iOS, EncryptedSharedPreferences on Android. See [`lib/app/data/datasources/local/secure_storage.dart`](../lib/app/data/datasources/local/secure_storage.dart).

### Transport
- `Dio` is configured with a bearer interceptor and a 401 refresh flow in [`lib/app/data/datasources/remote/dio_client.dart`](../lib/app/data/datasources/remote/dio_client.dart).
- All requests go over HTTPS in production (enforced by the API, not the client).

### Screenshot / screen-recording block
Sensitive screens mix in [`SecureScreenMixin`](../lib/app/core/utils/secure_screen.dart). On Android this sets `FLAG_SECURE` — the OS blocks screenshots, screen recording, and hides the window from the recent-apps preview.

Currently applied to:
- `two_factor_view.dart` — TOTP code entry
- `reset_password_view.dart` — new password entry
- `verify_email_view.dart` — verification landing

**iOS:** `FLAG_SECURE` has no iOS equivalent. The usual approach is a blur overlay on `AppLifecycleState.inactive`. Not implemented here — it's app-specific and the wrong default for every product. If you need it, wrap your root in a lifecycle listener that pushes a blur layer on `inactive` / `paused`.

### Clipboard wipe on 2FA
After a successful 2FA verify, the clipboard is cleared so a pasted TOTP code can't be read by another app. See [`two_factor_view.dart`](../lib/app/modules/auth/two_factor/two_factor_view.dart).

### Autofill + password manager integration
Auth forms use `AutofillGroup` + `AutofillHints` so the OS password manager saves credentials as a pair. `newPassword` hint on sign-up + reset screens triggers strong-password suggestions.

## What you need to decide

### Release obfuscation
Dart code is not obfuscated by default. For release builds:

```bash
flutter build apk --release --obfuscate --split-debug-info=build/symbols
flutter build ipa --release --obfuscate --split-debug-info=build/symbols
```

Keep `build/symbols/` — you need it to symbolicate crash reports. Don't commit it.

### Certificate pinning
Not enabled. If your threat model includes a hostile network (corporate MITM, public-WiFi attacker with a rogue CA), add pinning. `dio` supports this through a custom `HttpClientAdapter` — see the [dio docs](https://pub.dev/packages/dio#https). Pin the leaf or intermediate, rotate before it expires, and ship an override mechanism so an expired pin doesn't brick the app.

### `.env` in the release bundle
`.env` is declared as a Flutter asset so `flutter_dotenv` can read it. On release builds it ships inside the APK/IPA — anyone with a decompiler can read it. **Never put server secrets in `.env`.** Put only public values there: API base URL, public Firebase config, public analytics keys. Anything that would hurt if leaked belongs in the backend or in a remote-config service.

### Biometric lock
Not implemented. If you want Face ID / fingerprint to gate app entry or sensitive actions, add [`local_auth`](https://pub.dev/packages/local_auth) and gate the relevant routes. Keep in mind biometric auth is a UX gate, not a cryptographic one — it doesn't encrypt data by itself.

### Jailbreak / root detection
Not implemented. Most apps don't need it and the packages that do it are an arms race. Skip unless you have a specific compliance requirement.

## Dependencies + CVE scanning
Dependabot is configured in [`.github/dependabot.yml`](../.github/dependabot.yml) — pub weekly, GitHub Actions monthly. Review the PRs; don't auto-merge.

## Reporting
If you find a security issue in this template, open a private advisory on GitHub rather than a public issue.
