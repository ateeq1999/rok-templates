import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repositories/auth.repository.dart';
import '../../../core/errors/app_exception.dart';

part 'verify_email_state.dart';

class VerifyEmailCubit extends Cubit<VerifyEmailState> {
  VerifyEmailCubit({required AuthRepository authRepository})
      : _authRepository = authRepository,
        super(const VerifyEmailInitial());

  final AuthRepository _authRepository;

  Future<void> verify({required String token}) async {
    if (token.isEmpty) return;
    if (state is VerifyEmailLoading) return;
    emit(const VerifyEmailLoading());
    try {
      await _authRepository.verifyEmail(token: token);
      emit(const VerifyEmailSuccess());
    } on ApiException catch (e) {
      emit(VerifyEmailFailure(e.message));
    } catch (e) {
      emit(VerifyEmailFailure(e.toString()));
    }
  }

  Future<void> resendEmail() async {
    if (state is VerifyEmailLoading) return;
    emit(const VerifyEmailLoading());
    try {
      await _authRepository.sendVerificationEmail();
      emit(const VerifyEmailResent());
    } on ApiException catch (e) {
      emit(VerifyEmailFailure(e.message));
    } catch (e) {
      emit(VerifyEmailFailure(e.toString()));
    }
  }
}
