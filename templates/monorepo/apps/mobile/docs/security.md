# Security

What this template does for you, what it doesn't, and what you need to decide before shipping.

## What's built in

### Token storage
The bearer token and the signed-in user live in `flutter_secure_storage` — Keychain on iOS, EncryptedSharedPreferences on Android. See [`lib/app/services/auth_service.dart`](../lib/app/services/auth_service.dart). The API has no refresh token: a `401` to the session's token signs out.

### Transport
- Every call goes through the generated client (`clients/dart`), which sends the bearer token only to the operations the API's contract marks secured.
- All requests go over HTTPS in production (an `https://` `API_URL`; enforced by the API, not the client).

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
Not enabled. If your threat model includes a hostile network (corporate MITM, public-WiFi attacker with a rogue CA), add pinning. The generated client takes an `http.Client` (`ApiClient(httpClient: ...)`): give it an `IOClient` over an `HttpClient` whose `SecurityContext` trusts only your certificate. Pin the leaf or intermediate, rotate before it expires, and ship an override mechanism so an expired pin doesn't brick the app.

### Build-time settings in the release bundle
`--dart-define` values (`API_URL`, the flags, whatever `.env` holds for `--dart-define-from-file`) are compiled into the App — anyone with a decompiler can read them. **Never put server secrets there.** Put only public values there: the API's URL, public analytics keys. Anything that would hurt if leaked belongs in the backend.

### Biometric lock
Not implemented. If you want Face ID / fingerprint to gate app entry or sensitive actions, add [`local_auth`](https://pub.dev/packages/local_auth) and gate the relevant routes. Keep in mind biometric auth is a UX gate, not a cryptographic one — it doesn't encrypt data by itself.

### Jailbreak / root detection
Not implemented. Most apps don't need it and the packages that do it are an arms race. Skip unless you have a specific compliance requirement.

## Reporting
If you find a security issue in this template, open a private advisory on GitHub rather than a public issue.
