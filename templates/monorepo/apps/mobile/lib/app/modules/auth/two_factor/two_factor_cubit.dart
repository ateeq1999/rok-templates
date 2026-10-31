import 'package:api_client/api_client.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repositories/auth.repository.dart';

part 'two_factor_state.dart';

/// A two-factor sign-in's second step: the authenticator's code, with the
/// pending token signing in answered.
class TwoFactorCubit extends Cubit<TwoFactorState> {
  TwoFactorCubit({
    required AuthRepository authRepository,
    required String pendingToken,
  })  : _authRepository = authRepository,
        _pendingToken = pendingToken,
        super(const TwoFactorInitial());

  final AuthRepository _authRepository;
  final String _pendingToken;

  Future<void> verify({required String code}) async {
    if (code.length != 6) return;
    if (state is TwoFactorLoading) return;
    emit(const TwoFactorLoading());
    try {
      final signedIn = await _authRepository.verifyTwoFactor(
        pendingToken: _pendingToken,
        code: code,
      );
      emit(TwoFactorSuccess(token: signedIn.token, user: signedIn.user));
    } on ApiException catch (e) {
      emit(TwoFactorFailure(e.message));
    } catch (e) {
      emit(TwoFactorFailure(e.toString()));
    }
  }

  void reset() => emit(const TwoFactorInitial());
}
