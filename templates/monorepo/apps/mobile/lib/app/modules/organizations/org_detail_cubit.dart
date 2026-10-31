import 'package:flutter_bloc/flutter_bloc.dart';

import '../../data/models/organization.model.dart';
import '../../data/models/org_member.model.dart';
import '../../data/repositories/organization.repository.dart';
import '../../core/errors/app_exception.dart';

part 'org_detail_state.dart';

class OrgDetailCubit extends Cubit<OrgDetailState> {
  OrgDetailCubit({required OrganizationRepository orgRepository})
      : _orgRepository = orgRepository,
        super(const OrgDetailInitial());

  final OrganizationRepository _orgRepository;

  Future<void> load(String orgId) async {
    if (state is OrgDetailLoading) return;
    emit(const OrgDetailLoading());
    try {
      final results = await Future.wait([
        _orgRepository.getOrg(orgId),
        _orgRepository.listMembers(orgId),
      ]);
      emit(OrgDetailLoaded(
        org: results[0] as Organization,
        members: results[1] as List<OrgMember>,
      ));
    } on ApiException catch (e) {
      emit(OrgDetailFailure(e.message));
    } catch (e) {
      emit(OrgDetailFailure(e.toString()));
    }
  }

  Future<void> updateOrg(String name) async {
    final current = state;
    if (current is! OrgDetailLoaded) return;
    try {
      final updated = await _orgRepository.updateOrg(current.org.id, name);
      emit(current.copyWith(org: updated));
    } on ApiException catch (e) {
      emit(OrgDetailFailure(e.message));
    } catch (e) {
      emit(OrgDetailFailure(e.toString()));
    }
  }

  Future<void> addMember(String userId, String role) async {
    final current = state;
    if (current is! OrgDetailLoaded) return;
    try {
      final member =
          await _orgRepository.addMember(current.org.id, userId, role);
      emit(current.copyWith(members: [...current.members, member]));
    } on ApiException catch (e) {
      emit(OrgDetailFailure(e.message));
    } catch (e) {
      emit(OrgDetailFailure(e.toString()));
    }
  }

  Future<void> updateMemberRole(String userId, String role) async {
    final current = state;
    if (current is! OrgDetailLoaded) return;
    try {
      final updated =
          await _orgRepository.updateMemberRole(current.org.id, userId, role);
      final members = current.members
          .map((m) => m.userId == userId ? updated : m)
          .toList();
      emit(current.copyWith(members: members));
    } on ApiException catch (e) {
      emit(OrgDetailFailure(e.message));
    } catch (e) {
      emit(OrgDetailFailure(e.toString()));
    }
  }

  Future<void> removeMember(String userId) async {
    final current = state;
    if (current is! OrgDetailLoaded) return;
    try {
      await _orgRepository.removeMember(current.org.id, userId);
      final members =
          current.members.where((m) => m.userId != userId).toList();
      emit(current.copyWith(members: members));
    } on ApiException catch (e) {
      emit(OrgDetailFailure(e.message));
    } catch (e) {
      emit(OrgDetailFailure(e.toString()));
    }
  }
}
