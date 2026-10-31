import 'package:dio/dio.dart';

class AdminProvider {
  AdminProvider(this._dio);

  final Dio _dio;

  Future<Response<dynamic>> listUsers({
    int limit = 20,
    String? cursor,
    String? search,
    String? role,
    bool? verified,
    CancelToken? cancelToken,
  }) {
    final params = <String, dynamic>{'limit': limit};
    if (cursor != null) params['cursor'] = cursor;
    if (search != null && search.isNotEmpty) params['search'] = search;
    if (role != null) params['role'] = role;
    if (verified != null) params['verified'] = verified;
    return _dio.get(
      '/v1/api/admin/users',
      queryParameters: params,
      cancelToken: cancelToken,
    );
  }

  Future<Response<dynamic>> getStats({CancelToken? cancelToken}) =>
      _dio.get('/v1/api/admin/users/stats', cancelToken: cancelToken);

  Future<Response<dynamic>> getUser(String id) =>
      _dio.get('/v1/api/admin/users/$id');

  Future<Response<dynamic>> updateUser(String id, Map<String, dynamic> data) =>
      _dio.patch('/v1/api/admin/users/$id', data: data);

  Future<Response<dynamic>> deleteUser(String id) =>
      _dio.delete('/v1/api/admin/users/$id');

  Future<Response<dynamic>> listAuditLogs({
    String? userId,
    int limit = 20,
    String? cursor,
    CancelToken? cancelToken,
  }) {
    final params = <String, dynamic>{'limit': limit};
    if (userId != null) params['userId'] = userId;
    if (cursor != null) params['cursor'] = cursor;
    return _dio.get(
      '/v1/api/admin/audit-logs',
      queryParameters: params,
      cancelToken: cancelToken,
    );
  }
}
