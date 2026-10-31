import 'package:dio/dio.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../data/models/admin_user.model.dart';
import '../../data/repositories/admin.repository.dart';
import '../../core/errors/app_exception.dart';

part 'admin_users_state.dart';

class AdminUsersCubit extends Cubit<AdminUsersState> {
  AdminUsersCubit({required AdminRepository adminRepository})
      : _adminRepository = adminRepository,
        super(const AdminUsersInitial());

  final AdminRepository _adminRepository;
  CancelToken? _cancelToken;

  CancelToken _replaceCancelToken() {
    _cancelToken?.cancel('Superseded by a newer request');
    return _cancelToken = CancelToken();
  }

  Future<void> loadUsers({String search = ''}) async {
    if (state is AdminUsersLoading) return;
    emit(const AdminUsersLoading());
    final token = _replaceCancelToken();
    try {
      final result = await _adminRepository.listUsers(
        search: search,
        cancelToken: token,
      );
      emit(AdminUsersLoaded(
        users: result.users,
        cursor: result.cursor,
        hasMore: result.hasMore,
        search: search,
      ));
    } on ApiException catch (e) {
      if (token.isCancelled) return;
      emit(AdminUsersFailure(e.message));
    } catch (e) {
      if (token.isCancelled) return;
      emit(AdminUsersFailure(e.toString()));
    }
  }

  Future<void> loadMore() async {
    final current = state;
    if (current is! AdminUsersLoaded || !current.hasMore) return;
    final token = _replaceCancelToken();
    try {
      final result = await _adminRepository.listUsers(
        cursor: current.cursor,
        search: current.search,
        cancelToken: token,
      );
      emit(current.copyWith(
        users: [...current.users, ...result.users],
        cursor: result.cursor,
        hasMore: result.hasMore,
      ));
    } on ApiException catch (e) {
      if (token.isCancelled) return;
      emit(AdminUsersFailure(e.message));
    } catch (e) {
      if (token.isCancelled) return;
      emit(AdminUsersFailure(e.toString()));
    }
  }

  @override
  Future<void> close() {
    _cancelToken?.cancel('Cubit closed');
    return super.close();
  }

  Future<void> updateUser(String id, Map<String, dynamic> data) async {
    try {
      final updated = await _adminRepository.updateUser(id, data);
      final current = state;
      if (current is AdminUsersLoaded) {
        final users = current.users
            .map((u) => u.id == id ? updated : u)
            .toList();
        emit(current.copyWith(users: users));
      }
    } on ApiException catch (e) {
      emit(AdminUsersFailure(e.message));
    } catch (e) {
      emit(AdminUsersFailure(e.toString()));
    }
  }

  Future<void> deleteUser(String id) async {
    try {
      await _adminRepository.deleteUser(id);
      final current = state;
      if (current is AdminUsersLoaded) {
        emit(current.copyWith(
          users: current.users.where((u) => u.id != id).toList(),
        ));
      }
    } on ApiException catch (e) {
      emit(AdminUsersFailure(e.message));
    } catch (e) {
      emit(AdminUsersFailure(e.toString()));
    }
  }
}
