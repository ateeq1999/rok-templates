import 'package:flutter_bloc/flutter_bloc.dart';

import '../../data/models/admin_stats.model.dart';
import '../../data/repositories/admin.repository.dart';
import '../../core/errors/app_exception.dart';

part 'admin_stats_state.dart';

class AdminStatsCubit extends Cubit<AdminStatsState> {
  AdminStatsCubit({required AdminRepository adminRepository})
      : _adminRepository = adminRepository,
        super(const AdminStatsInitial());

  final AdminRepository _adminRepository;

  Future<void> loadStats() async {
    if (state is AdminStatsLoading) return;
    emit(const AdminStatsLoading());
    try {
      final stats = await _adminRepository.getStats();
      emit(AdminStatsLoaded(stats));
    } on ApiException catch (e) {
      emit(AdminStatsFailure(e.message));
    } catch (e) {
      emit(AdminStatsFailure(e.toString()));
    }
  }
}
