import 'package:dio/dio.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../data/models/audit_log.model.dart';
import '../../data/repositories/admin.repository.dart';
import '../../core/errors/app_exception.dart';

part 'audit_logs_state.dart';

class AuditLogsCubit extends Cubit<AuditLogsState> {
  AuditLogsCubit({required AdminRepository adminRepository})
      : _adminRepository = adminRepository,
        super(const AuditLogsInitial());

  final AdminRepository _adminRepository;
  CancelToken? _cancelToken;

  CancelToken _replaceCancelToken() {
    _cancelToken?.cancel('Superseded by a newer request');
    return _cancelToken = CancelToken();
  }

  Future<void> loadLogs({String? userId}) async {
    if (state is AuditLogsLoading) return;
    emit(const AuditLogsLoading());
    final token = _replaceCancelToken();
    try {
      final result = await _adminRepository.listAuditLogs(
        userId: userId,
        cancelToken: token,
      );
      emit(AuditLogsLoaded(
        logs: result.logs,
        cursor: result.cursor,
        hasMore: result.hasMore,
        userId: userId,
      ));
    } on ApiException catch (e) {
      if (token.isCancelled) return;
      emit(AuditLogsFailure(e.message));
    } catch (e) {
      if (token.isCancelled) return;
      emit(AuditLogsFailure(e.toString()));
    }
  }

  Future<void> loadMore() async {
    final current = state;
    if (current is! AuditLogsLoaded || !current.hasMore) return;
    final token = _replaceCancelToken();
    try {
      final result = await _adminRepository.listAuditLogs(
        userId: current.userId,
        cursor: current.cursor,
        cancelToken: token,
      );
      emit(current.copyWith(
        logs: [...current.logs, ...result.logs],
        cursor: result.cursor,
        hasMore: result.hasMore,
      ));
    } on ApiException catch (e) {
      if (token.isCancelled) return;
      emit(AuditLogsFailure(e.message));
    } catch (e) {
      if (token.isCancelled) return;
      emit(AuditLogsFailure(e.toString()));
    }
  }

  @override
  Future<void> close() {
    _cancelToken?.cancel('Cubit closed');
    return super.close();
  }
}
