part of 'admin_stats_cubit.dart';

sealed class AdminStatsState {
  const AdminStatsState();
}

final class AdminStatsInitial extends AdminStatsState {
  const AdminStatsInitial();
}

final class AdminStatsLoading extends AdminStatsState {
  const AdminStatsLoading();
}

final class AdminStatsLoaded extends AdminStatsState {
  const AdminStatsLoaded(this.stats);
  final AdminStats stats;
}

final class AdminStatsFailure extends AdminStatsState {
  const AdminStatsFailure(this.message);
  final String message;
}
