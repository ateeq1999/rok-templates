/// Build-time feature toggles: `--dart-define`s, or the `.env` file
/// `flutter run --dart-define-from-file=.env` reads.
///
/// Falsy values: `false`, `0`, `no`, `off`. Anything else (or absent) → `true`.
/// Add new flags here + in `.env.example` + consume via `context.read<FeatureFlags>()`.
///
/// Only features `apps/api` serves have a flag. Organizations, magic-link
/// sign-in, the admin panel, deleting the account, OAuth and push
/// notifications are not in the App until the API serves them (see
/// `docs/feature-flags.md`).
class FeatureFlags {
  const FeatureFlags({
    required this.twoFactor,
    required this.signUp,
    required this.themeCustomization,
  });

  final bool twoFactor;
  final bool signUp;
  final bool themeCustomization;

  static const _twoFactor = String.fromEnvironment('FEATURE_TWO_FACTOR');
  static const _signUp = String.fromEnvironment('FEATURE_SIGN_UP');
  static const _themeCustomization = String.fromEnvironment(
    'FEATURE_THEME_CUSTOMIZATION',
  );

  /// The flags the App was built with.
  factory FeatureFlags.fromEnvironment() {
    return FeatureFlags(
      twoFactor: parse(_twoFactor),
      signUp: parse(_signUp),
      themeCustomization: parse(_themeCustomization),
    );
  }

  /// A flag's value: absent or empty is `defaultValue`.
  static bool parse(String raw, {bool defaultValue = true}) {
    final value = raw.trim().toLowerCase();
    if (value.isEmpty) return defaultValue;
    return !{'false', '0', 'no', 'off'}.contains(value);
  }
}
