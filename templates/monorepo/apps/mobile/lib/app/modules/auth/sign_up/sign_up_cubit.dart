import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repositories/auth.repository.dart';
import '../../../core/errors/app_exception.dart';

part 'sign_up_state.dart';

class SignUpCubit extends Cubit<SignUpState> {
  SignUpCubit({required AuthRepository authRepository})
      : _authRepository = authRepository,
        super(const SignUpInitial());

  final AuthRepository _authRepository;

  bool _isPasswordHidden = true;
  bool get isPasswordHidden => _isPasswordHidden;

  void togglePasswordVisibility() {
    _isPasswordHidden = !_isPasswordHidden;
    emit(state);
  }

  Future<void> signUp({
    required String name,
    required String email,
    required String password,
  }) async {
    if (state is SignUpLoading) return;
    emit(const SignUpLoading());
    try {
      await _authRepository.signUp(
        name: name.trim(),
        email: email.trim(),
        password: password,
      );
      emit(const SignUpSuccess());
    } on ApiException catch (e) {
      emit(SignUpFailure(e.message, fieldErrors: e.fieldErrors));
    } catch (e) {
      emit(SignUpFailure(e.toString()));
    }
  }

  void reset() => emit(const SignUpInitial());
}
