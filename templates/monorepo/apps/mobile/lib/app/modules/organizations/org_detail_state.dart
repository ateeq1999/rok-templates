part of 'org_detail_cubit.dart';

sealed class OrgDetailState {
  const OrgDetailState();
}

final class OrgDetailInitial extends OrgDetailState {
  const OrgDetailInitial();
}

final class OrgDetailLoading extends OrgDetailState {
  const OrgDetailLoading();
}

final class OrgDetailLoaded extends OrgDetailState {
  const OrgDetailLoaded({required this.org, required this.members});
  final Organization org;
  final List<OrgMember> members;

  OrgDetailLoaded copyWith({Organization? org, List<OrgMember>? members}) {
    return OrgDetailLoaded(
      org: org ?? this.org,
      members: members ?? this.members,
    );
  }
}

final class OrgDetailFailure extends OrgDetailState {
  const OrgDetailFailure(this.message);
  final String message;
}
