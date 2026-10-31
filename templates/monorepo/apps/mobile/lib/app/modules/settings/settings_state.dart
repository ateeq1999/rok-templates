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

final class SettingsTwoFactorEnabled extends SettingsState {
  const SettingsTwoFactorEnabled({
    required this.isDarkMode,
    required this.uri,
    required this.qrCode,
  });
  final bool isDarkMode;
  final String uri;
  final String qrCode;
}

final class SettingsTwoFactorDisabled extends SettingsState {
  const SettingsTwoFactorDisabled({required this.isDarkMode});
  final bool isDarkMode;
}

final class SettingsTwoFactorVerified extends SettingsState {
  const SettingsTwoFactorVerified({required this.isDarkMode});
  final bool isDarkMode;
}

final class SettingsAccountDeleted extends SettingsState {
  const SettingsAccountDeleted({required this.isDarkMode});
  final bool isDarkMode;
}
