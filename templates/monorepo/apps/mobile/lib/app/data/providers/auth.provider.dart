import 'package:dio/dio.dart';

class AuthProvider {
  AuthProvider(this._dio);

  final Dio _dio;

  // BUG-U1: was /api/auth/sign-in/email; token=true makes API return Bearer token
  Future<Response<dynamic>> signIn({
    required String email,
    required String password,
  }) =>
      _dio.post(
        '/v1/api/auth/sign-in',
        queryParameters: {'token': 'true'},
        data: {'email': email, 'password': password},
      );

  // BUG-U2: was /api/auth/sign-up/email
  Future<Response<dynamic>> signUp({
    required String email,
    required String password,
    required String name,
  }) =>
      _dio.post(
        '/v1/api/auth/sign-up',
        data: {'email': email, 'password': password, 'name': name},
      );

  // BUG-U7: was /api/auth/sign-out
  Future<Response<dynamic>> signOut() =>
      _dio.post('/v1/api/auth/sign-out');

  // BUG-U3: was /api/auth/forgot-password (wrong spelling too)
  Future<Response<dynamic>> forgotPassword({required String email}) =>
      _dio.post('/v1/api/auth/forget-password', data: {'email': email});

  // BUG-U7: was /api/auth/reset-password
  Future<Response<dynamic>> resetPassword({
    required String token,
    required String password,
  }) =>
      _dio.post(
        '/v1/api/auth/reset-password',
        data: {'token': token, 'newPassword': password},
      );

  // BUG-U7: was /api/auth/change-password
  Future<Response<dynamic>> changePassword({
    required String currentPassword,
    required String newPassword,
  }) =>
      _dio.post(
        '/v1/api/auth/change-password',
        data: {'currentPassword': currentPassword, 'newPassword': newPassword},
      );

  // BUG-U7: was /api/auth/verify-email
  Future<Response<dynamic>> verifyEmail({required String token}) =>
      _dio.post('/v1/api/auth/verify-email', data: {'token': token});

  Future<Response<dynamic>> sendVerificationEmail() =>
      _dio.post('/v1/api/auth/send-verification-email');

  // BUG-U4: was /api/auth/two-factor/verify
  Future<Response<dynamic>> verifyTwoFactor({required String code}) =>
      _dio.post('/v1/api/auth/two-factor/verify-totp', data: {'code': code});

  Future<Response<dynamic>> enableTwoFactor() =>
      _dio.post('/v1/api/auth/two-factor/enable');

  Future<Response<dynamic>> verifyTwoFactorSetup({required String code}) =>
      _dio.post('/v1/api/auth/two-factor/verify-totp', data: {'code': code});

  Future<Response<dynamic>> disableTwoFactor({required String code}) =>
      _dio.post('/v1/api/auth/two-factor/disable', data: {'code': code});

  Future<Response<dynamic>> sendMagicLink({required String email}) =>
      _dio.post(
        '/v1/api/auth/magic-link/send-magic-link',
        data: {'email': email},
      );

  Future<Response<dynamic>> verifyMagicLink({required String token}) =>
      _dio.get(
        '/v1/api/auth/magic-link/verify-magic-link',
        queryParameters: {'token': token},
      );

  // BUG-U7: was /api/auth/token/refresh
  Future<Response<dynamic>> refreshToken() =>
      _dio.post('/v1/api/auth/token/refresh');
}
