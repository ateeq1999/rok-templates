part of 'audit_logs_cubit.dart';

sealed class AuditLogsState {
  const AuditLogsState();
}

final class AuditLogsInitial extends AuditLogsState {
  const AuditLogsInitial();
}

final class AuditLogsLoading extends AuditLogsState {
  const AuditLogsLoading();
}

final class AuditLogsLoaded extends AuditLogsState {
  const AuditLogsLoaded({
    required this.logs,
    this.cursor,
    required this.hasMore,
    this.userId,
  });

  final List<AuditLog> logs;
  final String? cursor;
  final bool hasMore;
  final String? userId;

  AuditLogsLoaded copyWith({
    List<AuditLog>? logs,
    String? cursor,
    bool? hasMore,
  }) {
    return AuditLogsLoaded(
      logs: logs ?? this.logs,
      cursor: cursor,
      hasMore: hasMore ?? this.hasMore,
      userId: userId,
    );
  }
}

final class AuditLogsFailure extends AuditLogsState {
  const AuditLogsFailure(this.message);
  final String message;
}
