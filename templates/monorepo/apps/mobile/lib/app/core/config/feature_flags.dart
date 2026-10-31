import 'package:flutter_dotenv/flutter_dotenv.dart';

/// Compile-time feature toggles loaded from `.env`.
///
/// Falsy values: `false`, `0`, `no`, `off`. Anything else (or absent) → `true`.
/// Add new flags here + in `.env.example` + consume via `context.read<FeatureFlags>()`.
class FeatureFlags {
  const FeatureFlags({
    required this.magicLink,
    required this.twoFactor,
    required this.organizations,
    required this.admin,
    required this.signUp,
    required this.oauth,
    required this.notifications,
    required this.themeCustomization,
    required this.deleteAccount,
  });

  final bool magicLink;
  final bool twoFactor;
  final bool organizations;
  final bool admin;
  final bool signUp;
  final bool oauth;
  final bool notifications;
  final bool themeCustomization;
  final bool deleteAccount;

  factory FeatureFlags.fromEnv() {
    return FeatureFlags(
      magicLink: _flag('FEATURE_MAGIC_LINK'),
      twoFactor: _flag('FEATURE_TWO_FACTOR'),
      organizations: _flag('FEATURE_ORGANIZATIONS'),
      admin: _flag('FEATURE_ADMIN'),
      signUp: _flag('FEATURE_SIGN_UP'),
      oauth: _flag('FEATURE_OAUTH', defaultValue: false),
      notifications: _flag('FEATURE_PUSH_NOTIFICATIONS'),
      themeCustomization: _flag('FEATURE_THEME_CUSTOMIZATION'),
      deleteAccount: _flag('FEATURE_DELETE_ACCOUNT', defaultValue: false),
    );
  }

  static bool _flag(String key, {bool defaultValue = true}) {
    final raw = dotenv.env[key]?.trim().toLowerCase();
    if (raw == null || raw.isEmpty) return defaultValue;
    return !{'false', '0', 'no', 'off'}.contains(raw);
  }
}
