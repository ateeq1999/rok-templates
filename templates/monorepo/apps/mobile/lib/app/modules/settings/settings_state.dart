part of 'settings_cubit.dart';

sealed class SettingsState {
  const SettingsState();
}

final class SettingsInitial extends SettingsState {
  const SettingsInitial({this.isDarkMode = false});
  final bool isDarkMode;
}

final class SettingsLoading extends SettingsState {
  const SettingsLoading({required this.isDarkMode});
  final bool isDarkMode;
}

final class SettingsPasswordChanged extends SettingsState {
  const SettingsPasswordChanged({required this.isDarkMode});
  final bool isDarkMode;
}

final class SettingsFailure extends SettingsState {
  const SettingsFailure(this.message, {required this.isDarkMode});
  final String message;
  final bool isDarkMode;
}

/// Two-factor set up, not on yet: the authenticator secret to confirm
/// with a first code.
final class SettingsTwoFactorSetUp extends SettingsState {
  const SettingsTwoFactorSetUp({
    required this.isDarkMode,
    required this.uri,
    required this.secret,
  });
  final bool isDarkMode;

  /// The `otpauth://` URI an authenticator app reads.
  final String uri;

  /// The secret, for typing in by hand.
  final String secret;
}

final class SettingsTwoFactorDisabled extends SettingsState {
  const SettingsTwoFactorDisabled({required this.isDarkMode});
  final bool isDarkMode;
}

/// Two-factor is on: the one-time recovery codes to keep.
final class SettingsTwoFactorEnabled extends SettingsState {
  const SettingsTwoFactorEnabled({
    required this.isDarkMode,
    required this.recoveryCodes,
  });
  final bool isDarkMode;
  final List<String> recoveryCodes;
}
