import 'package:dio/dio.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../data/models/organization.model.dart';
import '../../data/repositories/organization.repository.dart';
import '../../core/errors/app_exception.dart';

part 'org_list_state.dart';

class OrgListCubit extends Cubit<OrgListState> {
  OrgListCubit({required OrganizationRepository orgRepository})
      : _orgRepository = orgRepository,
        super(const OrgListInitial());

  final OrganizationRepository _orgRepository;
  CancelToken? _cancelToken;

  Future<void> loadOrgs() async {
    if (state is OrgListLoading) return;
    emit(const OrgListLoading());
    _cancelToken?.cancel('Superseded by a newer request');
    final token = _cancelToken = CancelToken();
    try {
      final orgs = await _orgRepository.listOrgs(cancelToken: token);
      emit(OrgListLoaded(orgs));
    } on ApiException catch (e) {
      if (token.isCancelled) return;
      emit(OrgListFailure(e.message));
    } catch (e) {
      if (token.isCancelled) return;
      emit(OrgListFailure(e.toString()));
    }
  }

  @override
  Future<void> close() {
    _cancelToken?.cancel('Cubit closed');
    return super.close();
  }

  Future<void> createOrg(String name) async {
    try {
      final org = await _orgRepository.createOrg(name);
      final current = state;
      if (current is OrgListLoaded) {
        emit(current.withOrg(org));
      }
    } on ApiException catch (e) {
      emit(OrgListFailure(e.message));
    } catch (e) {
      emit(OrgListFailure(e.toString()));
    }
  }

  Future<void> deleteOrg(String id) async {
    try {
      await _orgRepository.deleteOrg(id);
      final current = state;
      if (current is OrgListLoaded) {
        emit(current.withoutOrg(id));
      }
    } on ApiException catch (e) {
      emit(OrgListFailure(e.message));
    } catch (e) {
      emit(OrgListFailure(e.toString()));
    }
  }
}
