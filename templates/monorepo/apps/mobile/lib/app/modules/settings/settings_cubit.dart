import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../../data/repositories/auth.repository.dart';
import '../../data/repositories/user.repository.dart';
import '../../services/auth_service.dart';
import '../../core/errors/app_exception.dart';
import '../auth/auth_bloc.dart';

part 'settings_state.dart';

class SettingsCubit extends Cubit<SettingsState> {
  SettingsCubit({
    required AuthRepository authRepository,
    required UserRepository userRepository,
    required AuthService authService,
    required AuthBloc authBloc,
    required SharedPreferences prefs,
  })  : _authRepository = authRepository,
        _userRepository = userRepository,
        _authService = authService,
        _authBloc = authBloc,
        _prefs = prefs,
        super(SettingsInitial(isDarkMode: prefs.getBool('isDarkMode') ?? false));

  final AuthRepository _authRepository;
  final UserRepository _userRepository;
  final AuthService _authService;
  final AuthBloc _authBloc;
  final SharedPreferences _prefs;

  bool get isDarkMode {
    final s = state;
    return switch (s) {
      SettingsInitial(:final isDarkMode) => isDarkMode,
      SettingsLoading(:final isDarkMode) => isDarkMode,
      SettingsPasswordChanged(:final isDarkMode) => isDarkMode,
      SettingsFailure(:final isDarkMode) => isDarkMode,
      SettingsTwoFactorEnabled(:final isDarkMode) => isDarkMode,
      SettingsTwoFactorDisabled(:final isDarkMode) => isDarkMode,
      SettingsTwoFactorVerified(:final isDarkMode) => isDarkMode,
      SettingsAccountDeleted(:final isDarkMode) => isDarkMode,
    };
  }

  void toggleTheme() {
    final newValue = !isDarkMode;
    _prefs.setBool('isDarkMode', newValue);
    emit(SettingsInitial(isDarkMode: newValue));
  }

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
    } on ApiException catch (e) {
      emit(SettingsFailure(e.message, isDarkMode: isDarkMode));
    } catch (e) {
      emit(SettingsFailure(e.toString(), isDarkMode: isDarkMode));
    }
  }

  Future<void> signOut() async {
    emit(SettingsLoading(isDarkMode: isDarkMode));
    try {
      await _authRepository.signOut();
    } catch (_) {
      // Best-effort — always clear local state.
    } finally {
      await _authService.signOut(); // BUG-08 fix: uses AppRoutes via AuthBloc
      _authBloc.add(const AuthSignedOut());
    }
  }

  Future<void> enableTwoFactor() async {
    emit(SettingsLoading(isDarkMode: isDarkMode));
    try {
      final result = await _authRepository.enableTwoFactor();
      emit(SettingsTwoFactorEnabled(
        isDarkMode: isDarkMode,
        uri: result.uri,
        qrCode: result.qrCode,
      ));
    } on ApiException catch (e) {
      emit(SettingsFailure(e.message, isDarkMode: isDarkMode));
    } catch (e) {
      emit(SettingsFailure(e.toString(), isDarkMode: isDarkMode));
    }
  }

  Future<void> verifyTwoFactorSetup({required String code}) async {
    emit(SettingsLoading(isDarkMode: isDarkMode));
    try {
      await _authRepository.verifyTwoFactorSetup(code: code);
      emit(SettingsTwoFactorVerified(isDarkMode: isDarkMode));
    } on ApiException catch (e) {
      emit(SettingsFailure(e.message, isDarkMode: isDarkMode));
    } catch (e) {
      emit(SettingsFailure(e.toString(), isDarkMode: isDarkMode));
    }
  }

  Future<void> disableTwoFactor({required String code}) async {
    emit(SettingsLoading(isDarkMode: isDarkMode));
    try {
      await _authRepository.disableTwoFactor(code: code);
      emit(SettingsTwoFactorDisabled(isDarkMode: isDarkMode));
    } on ApiException catch (e) {
      emit(SettingsFailure(e.message, isDarkMode: isDarkMode));
    } catch (e) {
      emit(SettingsFailure(e.toString(), isDarkMode: isDarkMode));
    }
  }

  Future<void> deleteAccount({String? password}) async {
    emit(SettingsLoading(isDarkMode: isDarkMode));
    try {
      await _userRepository.deleteAccount(password: password);
      emit(SettingsAccountDeleted(isDarkMode: isDarkMode));
      // Clear local auth state so GoRouter redirects to sign-in.
      await _authService.signOut();
      _authBloc.add(const AuthSignedOut());
    } on ApiException catch (e) {
      emit(SettingsFailure(e.message, isDarkMode: isDarkMode));
    } catch (e) {
      emit(SettingsFailure(e.toString(), isDarkMode: isDarkMode));
    }
  }

  void resetToInitial() => emit(SettingsInitial(isDarkMode: isDarkMode));
}
