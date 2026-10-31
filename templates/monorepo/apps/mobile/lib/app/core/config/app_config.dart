import 'package:flutter/foundation.dart';

/// Settings fixed when the App is built: `--dart-define`s, or the
/// `.env` file `flutter run --dart-define-from-file=.env` reads (see
/// `.env.example`).
abstract final class AppConfig {
  static const _apiUrl = String.fromEnvironment('API_URL');

  /// The API's origin. Without `API_URL`, `apps/api` on this machine
  /// (`127.0.0.1:8080`, its `BIND_ADDR`), which an Android emulator reaches
  /// as `10.0.2.2`.
  static String get apiUrl {
    if (_apiUrl.isNotEmpty) return _apiUrl;
    final android = !kIsWeb && defaultTargetPlatform == TargetPlatform.android;
    return android ? 'http://10.0.2.2:8080' : 'http://127.0.0.1:8080';
  }

  /// The deep-link scheme the App answers (`<scheme>://auth/...`).
  static const appScheme = String.fromEnvironment(
    'APP_SCHEME',
    defaultValue: 'myapp',
  );
}
