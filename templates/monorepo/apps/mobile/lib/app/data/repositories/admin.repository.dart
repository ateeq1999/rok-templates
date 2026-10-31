import 'package:dio/dio.dart';

import '../models/admin_user.model.dart';
import '../models/admin_stats.model.dart';
import '../models/audit_log.model.dart';
import '../providers/admin.provider.dart';
import '../../core/errors/app_exception.dart';
import '../../core/utils/response_parser.dart';

class AdminRepository {
  AdminRepository(this._provider);

  final AdminProvider _provider;

  Future<({List<AdminUser> users, String? cursor, bool hasMore})> listUsers({
    int limit = 20,
    String? cursor,
    String? search,
    String? role,
    bool? verified,
    CancelToken? cancelToken,
  }) async {
    try {
      final response = await _provider.listUsers(
        limit: limit,
        cursor: cursor,
        search: search,
        role: role,
        verified: verified,
        cancelToken: cancelToken,
      );
      final data = response.data as Map<String, dynamic>;
      final items = (data['data'] as List? ?? [])
          .cast<Map<String, dynamic>>()
          .map(AdminUser.fromJson)
          .toList();
      final pagination = _pagination(data);
      return (
        users: items,
        cursor: pagination.cursor,
        hasMore: pagination.hasMore,
      );
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<AdminStats> getStats({CancelToken? cancelToken}) async {
    try {
      final response = await _provider.getStats(cancelToken: cancelToken);
      return AdminStats.fromJson(
          unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<AdminUser> getUser(String id) async {
    try {
      final response = await _provider.getUser(id);
      return AdminUser.fromJson(unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<AdminUser> updateUser(String id, Map<String, dynamic> data) async {
    try {
      final response = await _provider.updateUser(id, data);
      return AdminUser.fromJson(unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> deleteUser(String id) async {
    try {
      await _provider.deleteUser(id);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<({List<AuditLog> logs, String? cursor, bool hasMore})> listAuditLogs({
    String? userId,
    int limit = 20,
    String? cursor,
    CancelToken? cancelToken,
  }) async {
    try {
      final response = await _provider.listAuditLogs(
        userId: userId,
        limit: limit,
        cursor: cursor,
        cancelToken: cancelToken,
      );
      final data = response.data as Map<String, dynamic>;
      final items = (data['data'] as List? ?? [])
          .cast<Map<String, dynamic>>()
          .map(AuditLog.fromJson)
          .toList();
      final pagination = _pagination(data);
      return (
        logs: items,
        cursor: pagination.cursor,
        hasMore: pagination.hasMore,
      );
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  /// Envelope `meta.pagination` shape:
  /// `{ limit, hasNextPage, nextCursor }`.
  /// BUG: previously read `meta.cursor` / `meta.hasMore` which never existed.
  static ({String? cursor, bool hasMore}) _pagination(
      Map<String, dynamic> envelope) {
    final meta = envelope['meta'] as Map<String, dynamic>? ?? const {};
    final p = meta['pagination'] as Map<String, dynamic>? ?? const {};
    return (
      cursor: p['nextCursor'] as String?,
      hasMore: p['hasNextPage'] as bool? ?? false,
    );
  }
}
