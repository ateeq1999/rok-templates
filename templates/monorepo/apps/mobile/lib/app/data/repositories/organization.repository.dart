import 'package:dio/dio.dart';

import '../models/organization.model.dart';
import '../models/org_member.model.dart';
import '../models/org_invitation.model.dart';
import '../providers/organization.provider.dart';
import '../../core/errors/app_exception.dart';
import '../../core/utils/response_parser.dart';

class OrganizationRepository {
  OrganizationRepository(this._provider);

  final OrganizationProvider _provider;

  Future<List<Organization>> listOrgs({CancelToken? cancelToken}) async {
    try {
      final response = await _provider.listOrgs(cancelToken: cancelToken);
      return unwrapEnvelopeList(response.data)
          .map(Organization.fromJson)
          .toList();
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<Organization> createOrg(String name) async {
    try {
      final response = await _provider.createOrg(name);
      return Organization.fromJson(unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<Organization> getOrg(String id) async {
    try {
      final response = await _provider.getOrg(id);
      return Organization.fromJson(unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<Organization> updateOrg(String id, String name) async {
    try {
      final response = await _provider.updateOrg(id, name);
      return Organization.fromJson(unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> deleteOrg(String id) async {
    try {
      await _provider.deleteOrg(id);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<List<OrgMember>> listMembers(String orgId,
      {CancelToken? cancelToken}) async {
    try {
      final response =
          await _provider.listMembers(orgId, cancelToken: cancelToken);
      return unwrapEnvelopeList(response.data)
          .map(OrgMember.fromJson)
          .toList();
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<OrgMember> addMember(
      String orgId, String userId, String role) async {
    try {
      final response = await _provider.addMember(orgId, userId, role);
      return OrgMember.fromJson(unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<OrgMember> updateMemberRole(
      String orgId, String userId, String role) async {
    try {
      final response =
          await _provider.updateMemberRole(orgId, userId, role);
      return OrgMember.fromJson(unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> removeMember(String orgId, String userId) async {
    try {
      await _provider.removeMember(orgId, userId);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<OrgInvitation> inviteMember(
      String orgId, String email, String role) async {
    try {
      final response = await _provider.inviteMember(orgId, email, role);
      return OrgInvitation.fromJson(unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<void> acceptInvitation(String token) async {
    try {
      await _provider.acceptInvitation(token);
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }
}
