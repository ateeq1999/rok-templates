import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repositories/auth.repository.dart';
import '../../../core/errors/app_exception.dart';

part 'forgot_password_state.dart';

class ForgotPasswordCubit extends Cubit<ForgotPasswordState> {
  ForgotPasswordCubit({required AuthRepository authRepository})
      : _authRepository = authRepository,
        super(const ForgotPasswordInitial());

  final AuthRepository _authRepository;

  Future<void> submit({required String email}) async {
    if (state is ForgotPasswordLoading) return;
    emit(const ForgotPasswordLoading());
    try {
      await _authRepository.forgotPassword(email: email.trim());
      emit(const ForgotPasswordSuccess());
    } on ApiException catch (e) {
      emit(ForgotPasswordFailure(e.message, fieldErrors: e.fieldErrors));
    } catch (e) {
      emit(ForgotPasswordFailure(e.toString()));
    }
  }

  void reset() => emit(const ForgotPasswordInitial());
}
