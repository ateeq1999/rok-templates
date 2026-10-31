import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repositories/auth.repository.dart';
import '../../../core/errors/app_exception.dart';

part 'reset_password_state.dart';

class ResetPasswordCubit extends Cubit<ResetPasswordState> {
  ResetPasswordCubit({required AuthRepository authRepository})
      : _authRepository = authRepository,
        super(const ResetPasswordInitial());

  final AuthRepository _authRepository;

  bool _isPasswordHidden = true;
  bool get isPasswordHidden => _isPasswordHidden;

  void togglePasswordVisibility() {
    _isPasswordHidden = !_isPasswordHidden;
    emit(state);
  }

  Future<void> reset({required String token, required String password}) async {
    if (state is ResetPasswordLoading) return;
    emit(const ResetPasswordLoading());
    try {
      await _authRepository.resetPassword(
        token: token,
        password: password.trim(), // BUG-17 fix: trim password
      );
      emit(const ResetPasswordSuccess());
    } on ApiException catch (e) {
      emit(ResetPasswordFailure(e.message, fieldErrors: e.fieldErrors));
    } catch (e) {
      emit(ResetPasswordFailure(e.toString()));
    }
  }

  void resetState() => emit(const ResetPasswordInitial());
}
