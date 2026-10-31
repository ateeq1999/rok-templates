import 'package:flutter_bloc/flutter_bloc.dart';

import '../../data/models/org_invitation.model.dart';
import '../../data/repositories/organization.repository.dart';
import '../../core/errors/app_exception.dart';

part 'org_invitation_state.dart';

class OrgInvitationCubit extends Cubit<OrgInvitationState> {
  OrgInvitationCubit({required OrganizationRepository orgRepository})
      : _orgRepository = orgRepository,
        super(const OrgInvitationInitial());

  final OrganizationRepository _orgRepository;

  Future<void> inviteMember(
      String orgId, String email, String role) async {
    if (state is OrgInvitationLoading) return;
    emit(const OrgInvitationLoading());
    try {
      final invitation =
          await _orgRepository.inviteMember(orgId, email, role);
      emit(OrgInvitationSent(invitation));
    } on ApiException catch (e) {
      emit(OrgInvitationFailure(e.message));
    } catch (e) {
      emit(OrgInvitationFailure(e.toString()));
    }
  }

  Future<void> acceptInvitation(String token) async {
    if (state is OrgInvitationLoading) return;
    emit(const OrgInvitationLoading());
    try {
      await _orgRepository.acceptInvitation(token);
      emit(const OrgInvitationAccepted());
    } on ApiException catch (e) {
      emit(OrgInvitationFailure(e.message));
    } catch (e) {
      emit(OrgInvitationFailure(e.toString()));
    }
  }
}
