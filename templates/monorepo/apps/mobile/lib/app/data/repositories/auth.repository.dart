import 'package:dio/dio.dart';

import '../models/auth_response.model.dart';
import '../providers/auth.provider.dart';
import '../../core/errors/app_exception.dart';

class AuthRepository {
  AuthRepository(this._provider);

  final AuthProvider _provider;

  Future<AuthResponse> signIn({
    required String email,
    required String password,
  }) async {
    try {
      final response = await _provider.signIn(email: email, password: password);
      return AuthResponse.fromJson(response.data as Map<String, dynamic>);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<AuthResponse> signUp({
    required String email,
    required String password,
    required String name,
  }) async {
    try {
      final response =
          await _provider.signUp(email: email, password: password, name: name);
      return AuthResponse.fromJson(response.data as Map<String, dynamic>);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> signOut() async {
    try {
      await _provider.signOut();
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> forgotPassword({required String email}) async {
    try {
      await _provider.forgotPassword(email: email);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> resetPassword({
    required String token,
    required String password,
  }) async {
    try {
      await _provider.resetPassword(token: token, password: password);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> changePassword({
    required String currentPassword,
    required String newPassword,
  }) async {
    try {
      await _provider.changePassword(
        currentPassword: currentPassword,
        newPassword: newPassword,
      );
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> verifyEmail({required String token}) async {
    try {
      await _provider.verifyEmail(token: token);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> sendVerificationEmail() async {
    try {
      await _provider.sendVerificationEmail();
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<AuthResponse> verifyTwoFactor({required String code}) async {
    try {
      final response = await _provider.verifyTwoFactor(code: code);
      return AuthResponse.fromJson(response.data as Map<String, dynamic>);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  /// Returns the TOTP URI (`otpauth://...`) and base64 QR code PNG.
  /// BUG-R2 fix: API returns `{qrCode, uri}` — was wrongly reading `totpUri`.
  Future<({String uri, String qrCode})> enableTwoFactor() async {
    try {
      final response = await _provider.enableTwoFactor();
      final data = response.data as Map<String, dynamic>;
      return (
        uri: data['uri'] as String,
        qrCode: data['qrCode'] as String? ?? '',
      );
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  /// Confirm 2FA setup by verifying the first TOTP code from the authenticator.
  Future<void> verifyTwoFactorSetup({required String code}) async {
    try {
      await _provider.verifyTwoFactorSetup(code: code);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> disableTwoFactor({required String code}) async {
    try {
      await _provider.disableTwoFactor(code: code);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> sendMagicLink({required String email}) async {
    try {
      await _provider.sendMagicLink(email: email);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<AuthResponse> verifyMagicLink({required String token}) async {
    try {
      final response = await _provider.verifyMagicLink(token: token);
      return AuthResponse.fromJson(response.data as Map<String, dynamic>);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }
}
