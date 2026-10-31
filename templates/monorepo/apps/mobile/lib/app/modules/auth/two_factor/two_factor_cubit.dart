import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/models/user.model.dart';
import '../../../data/repositories/auth.repository.dart';
import '../../../core/errors/app_exception.dart';

part 'two_factor_state.dart';

/// BUG-05 fix: actually calls the TOTP verification API instead of
/// silently succeeding with no network call.
class TwoFactorCubit extends Cubit<TwoFactorState> {
  TwoFactorCubit({required AuthRepository authRepository})
      : _authRepository = authRepository,
        super(const TwoFactorInitial());

  final AuthRepository _authRepository;

  Future<void> verify({required String code}) async {
    if (code.length != 6) return;
    if (state is TwoFactorLoading) return;
    emit(const TwoFactorLoading());
    try {
      final response = await _authRepository.verifyTwoFactor(code: code);
      emit(TwoFactorSuccess(token: response.token, user: response.user));
    } on ApiException catch (e) {
      emit(TwoFactorFailure(e.message));
    } catch (e) {
      emit(TwoFactorFailure(e.toString()));
    }
  }

  void reset() => emit(const TwoFactorInitial());
}
