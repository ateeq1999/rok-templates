part of 'org_list_cubit.dart';

sealed class OrgListState {
  const OrgListState();
}

final class OrgListInitial extends OrgListState {
  const OrgListInitial();
}

final class OrgListLoading extends OrgListState {
  const OrgListLoading();
}

final class OrgListLoaded extends OrgListState {
  const OrgListLoaded(this.orgs);
  final List<Organization> orgs;

  OrgListLoaded withOrg(Organization org) =>
      OrgListLoaded([...orgs, org]);

  OrgListLoaded withoutOrg(String id) =>
      OrgListLoaded(orgs.where((o) => o.id != id).toList());
}

final class OrgListFailure extends OrgListState {
  const OrgListFailure(this.message);
  final String message;
}
