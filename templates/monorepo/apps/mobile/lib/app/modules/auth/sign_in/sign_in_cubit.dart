import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/models/user.model.dart';
import '../../../data/repositories/auth.repository.dart';
import '../../../core/errors/app_exception.dart';

part 'sign_in_state.dart';

class SignInCubit extends Cubit<SignInState> {
  SignInCubit({required AuthRepository authRepository})
      : _authRepository = authRepository,
        super(const SignInInitial());

  final AuthRepository _authRepository;

  bool _isPasswordHidden = true;
  bool get isPasswordHidden => _isPasswordHidden;

  void togglePasswordVisibility() {
    _isPasswordHidden = !_isPasswordHidden;
    // Re-emit current state to trigger BlocBuilder rebuild.
    emit(state);
  }

  Future<void> signIn({required String email, required String password}) async {
    if (state is SignInLoading) return; // BUG-16 fix: debounce rapid taps
    emit(const SignInLoading());
    try {
      final response = await _authRepository.signIn(
        email: email.trim(),
        password: password,
      );
      // FL5.3: API returns twoFactorEnabled=true with no token → 2FA required.
      if (response.token == null && response.user.twoFactorEnabled) {
        emit(const SignInTwoFactorRequired());
        return;
      }
      emit(SignInSuccess(token: response.token, user: response.user));
    } on ApiException catch (e) {
      emit(SignInFailure(e.message, fieldErrors: e.fieldErrors));
    } catch (e) {
      emit(SignInFailure(e.toString()));
    }
  }

  void reset() => emit(const SignInInitial());
}
