import 'package:api_client/api_client.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../../data/repositories/auth.repository.dart';
import '../../data/repositories/user.repository.dart';
import '../auth/auth_bloc.dart';

part 'settings_state.dart';

class SettingsCubit extends Cubit<SettingsState> {
  SettingsCubit({
    required AuthRepository authRepository,
    required UserRepository userRepository,
    required AuthBloc authBloc,
    required SharedPreferences prefs,
  })  : _authRepository = authRepository,
        _userRepository = userRepository,
        _authBloc = authBloc,
        _prefs = prefs,
        super(SettingsInitial(isDarkMode: prefs.getBool('isDarkMode') ?? false));

  final AuthRepository _authRepository;
  final UserRepository _userRepository;
  final AuthBloc _authBloc;
  final SharedPreferences _prefs;

  bool get isDarkMode {
    final s = state;
    return switch (s) {
      SettingsInitial(:final isDarkMode) => isDarkMode,
      SettingsLoading(:final isDarkMode) => isDarkMode,
      SettingsPasswordChanged(:final isDarkMode) => isDarkMode,
      SettingsFailure(:final isDarkMode) => isDarkMode,
      SettingsTwoFactorSetUp(:final isDarkMode) => isDarkMode,
      SettingsTwoFactorDisabled(:final isDarkMode) => isDarkMode,
      SettingsTwoFactorEnabled(:final isDarkMode) => isDarkMode,
    };
  }

  void toggleTheme() {
    final newValue = !isDarkMode;
    _prefs.setBool('isDarkMode', newValue);
    emit(SettingsInitial(isDarkMode: newValue));
  }

  /// Changes the password. The API signs every session out with it, so
  /// the App signs in again.
  Future<void> changePassword({
    required String currentPassword,
    required String newPassword,
  }) async {
    if (currentPassword.isEmpty || newPassword.isEmpty) return;
    emit(SettingsLoading(isDarkMode: isDarkMode));
    try {
      await _authRepository.changePassword(
        currentPassword: currentPassword,
        newPassword: newPassword,
      );
      emit(SettingsPasswordChanged(isDarkMode: isDarkMode));
      _authBloc.add(const AuthSignedOut());
    } on ApiException catch (e) {
      emit(SettingsFailure(e.message, isDarkMode: isDarkMode));
    } catch (e) {
      emit(SettingsFailure(e.toString(), isDarkMode: isDarkMode));
    }
  }

  Future<void> signOut() async {
    emit(SettingsLoading(isDarkMode: isDarkMode));
    await _authRepository.signOut();
    _authBloc.add(const AuthSignedOut());
  }

  /// Starts two-factor: a new authenticator secret to scan or type in.
  Future<void> setUpTwoFactor() async {
    emit(SettingsLoading(isDarkMode: isDarkMode));
    try {
      final setup = await _authRepository.setUpTwoFactor();
      emit(SettingsTwoFactorSetUp(
        isDarkMode: isDarkMode,
        uri: setup.provisioningUri,
        secret: setup.secret,
      ));
    } on ApiException catch (e) {
      emit(SettingsFailure(e.message, isDarkMode: isDarkMode));
    } catch (e) {
      emit(SettingsFailure(e.toString(), isDarkMode: isDarkMode));
    }
  }

  /// Turns two-factor on with the authenticator's first `code`.
  Future<void> enableTwoFactor({required String code}) async {
    emit(SettingsLoading(isDarkMode: isDarkMode));
    try {
      final recoveryCodes = await _authRepository.enableTwoFactor(code: code);
      await _refreshUser();
      emit(SettingsTwoFactorEnabled(
        isDarkMode: isDarkMode,
        recoveryCodes: recoveryCodes,
      ));
    } on ApiException catch (e) {
      emit(SettingsFailure(e.message, isDarkMode: isDarkMode));
    } catch (e) {
      emit(SettingsFailure(e.toString(), isDarkMode: isDarkMode));
    }
  }

  /// Turns two-factor off; the API asks for the password.
  Future<void> disableTwoFactor({required String currentPassword}) async {
    emit(SettingsLoading(isDarkMode: isDarkMode));
    try {
      await _authRepository.disableTwoFactor(currentPassword: currentPassword);
      await _refreshUser();
      emit(SettingsTwoFactorDisabled(isDarkMode: isDarkMode));
    } on ApiException catch (e) {
      emit(SettingsFailure(e.message, isDarkMode: isDarkMode));
    } catch (e) {
      emit(SettingsFailure(e.toString(), isDarkMode: isDarkMode));
    }
  }

  /// Reads the signed-in user again (its two-factor state changed).
  Future<void> _refreshUser() async {
    final authState = _authBloc.state;
    if (authState is! AuthAuthenticated) return;
    final user = await _userRepository.getMe();
    _authBloc.add(AuthUserChanged(user: user, token: authState.token));
  }

  void resetToInitial() => emit(SettingsInitial(isDarkMode: isDarkMode));
}
