import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_dotenv/flutter_dotenv.dart';

import 'auth_service.dart';

/// Dio HTTP client with a real AuthInterceptor.
///
/// • Attaches `Authorization: Bearer <token>` to every request (BUG-01 fix).
/// • On 401, attempts a token refresh and retries the original request (BUG-07 fix).
/// • If refresh fails, calls [AuthService.signOut] which broadcasts
///   [AuthStatus.unauthenticated] to the AuthBloc via the status stream.
class DioService {
  late final Dio _dio;

  Dio get dio => _dio;

  DioService(AuthService authService) {
    _dio = Dio(
      BaseOptions(
        baseUrl: dotenv.env['API_URL'] ?? 'http://10.0.2.2:5555',
        connectTimeout: const Duration(seconds: 30),
        receiveTimeout: const Duration(seconds: 30),
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json',
        },
      ),
    );

    _dio.interceptors.add(
      _AuthInterceptor(authService: authService, dio: _dio),
    );

    if (kDebugMode) {
      _dio.interceptors.add(
        LogInterceptor(
          requestBody: true,
          responseBody: true,
          logPrint: (o) => debugPrint(o.toString()),
        ),
      );
    }
  }
}

class _AuthInterceptor extends Interceptor {
  _AuthInterceptor({required this.authService, required this.dio});

  final AuthService authService;
  final Dio dio;

  /// Separate Dio instance used exclusively for token refresh to avoid
  /// triggering this interceptor recursively.
  late final Dio _refreshDio = Dio(
    BaseOptions(
      baseUrl: dotenv.env['API_URL'] ?? 'http://localhost:3000',
      connectTimeout: const Duration(seconds: 15),
      receiveTimeout: const Duration(seconds: 15),
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
      },
    ),
  );

  bool _isRefreshing = false;

  @override
  Future<void> onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    final token = await authService.getToken();
    if (token != null) {
      options.headers['Authorization'] = 'Bearer $token';
    }
    handler.next(options);
  }

  @override
  Future<void> onError(
    DioException err,
    ErrorInterceptorHandler handler,
  ) async {
    if (err.response?.statusCode == 401 && !_isRefreshing) {
      _isRefreshing = true;
      try {
        final refreshed = await _attemptRefresh();
        if (refreshed) {
          final newToken = await authService.getToken();
          err.requestOptions.headers['Authorization'] = 'Bearer $newToken';
          final retryResponse = await dio.fetch(err.requestOptions);
          _isRefreshing = false;
          return handler.resolve(retryResponse);
        }
      } catch (_) {
        // Refresh failed — fall through to sign out.
      }
      _isRefreshing = false;
      await authService.signOut();
    }
    handler.next(err);
  }

  Future<bool> _attemptRefresh() async {
    final token = await authService.getToken();
    if (token == null) return false;

    final response = await _refreshDio.post(
      '/v1/api/auth/token/refresh',
      options: Options(headers: {'Authorization': 'Bearer $token'}),
    );

    if (response.statusCode == 200 && response.data is Map) {
      final newToken = response.data['token'] as String?;
      if (newToken != null) {
        await authService.saveToken(newToken);
        return true;
      }
    }
    return false;
  }
}
