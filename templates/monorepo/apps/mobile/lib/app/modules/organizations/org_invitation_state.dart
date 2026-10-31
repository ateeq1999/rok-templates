part of 'org_invitation_cubit.dart';

sealed class OrgInvitationState {
  const OrgInvitationState();
}

final class OrgInvitationInitial extends OrgInvitationState {
  const OrgInvitationInitial();
}

final class OrgInvitationLoading extends OrgInvitationState {
  const OrgInvitationLoading();
}

final class OrgInvitationSent extends OrgInvitationState {
  const OrgInvitationSent(this.invitation);
  final OrgInvitation invitation;
}

final class OrgInvitationAccepted extends OrgInvitationState {
  const OrgInvitationAccepted();
}

final class OrgInvitationFailure extends OrgInvitationState {
  const OrgInvitationFailure(this.message);
  final String message;
}
