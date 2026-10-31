import 'package:dio/dio.dart';

class OrganizationProvider {
  OrganizationProvider(this._dio);

  final Dio _dio;

  Options _orgHeaders(String orgId) =>
      Options(headers: {'X-Organization-Id': orgId});

  Future<Response<dynamic>> listOrgs({CancelToken? cancelToken}) =>
      _dio.get('/v1/api/organizations', cancelToken: cancelToken);

  Future<Response<dynamic>> createOrg(String name) =>
      _dio.post('/v1/api/organizations', data: {'name': name});

  Future<Response<dynamic>> getOrg(String id) =>
      _dio.get('/v1/api/organizations/$id',
          options: _orgHeaders(id));

  Future<Response<dynamic>> updateOrg(String id, String name) =>
      _dio.patch('/v1/api/organizations/$id',
          data: {'name': name}, options: _orgHeaders(id));

  Future<Response<dynamic>> deleteOrg(String id) =>
      _dio.delete('/v1/api/organizations/$id', options: _orgHeaders(id));

  Future<Response<dynamic>> listMembers(String orgId,
          {CancelToken? cancelToken}) =>
      _dio.get('/v1/api/organizations/$orgId/members',
          options: _orgHeaders(orgId), cancelToken: cancelToken);

  Future<Response<dynamic>> addMember(
          String orgId, String userId, String role) =>
      _dio.post('/v1/api/organizations/$orgId/members',
          data: {'userId': userId, 'role': role},
          options: _orgHeaders(orgId));

  Future<Response<dynamic>> updateMemberRole(
          String orgId, String userId, String role) =>
      _dio.patch('/v1/api/organizations/$orgId/members/$userId',
          data: {'role': role}, options: _orgHeaders(orgId));

  Future<Response<dynamic>> removeMember(String orgId, String userId) =>
      _dio.delete('/v1/api/organizations/$orgId/members/$userId',
          options: _orgHeaders(orgId));

  Future<Response<dynamic>> inviteMember(
          String orgId, String email, String role) =>
      _dio.post('/v1/api/organizations/$orgId/invitations',
          data: {'email': email, 'role': role},
          options: _orgHeaders(orgId));

  Future<Response<dynamic>> acceptInvitation(String token) =>
      _dio.post('/v1/api/organizations/invitations/accept',
          data: {'token': token});
}
