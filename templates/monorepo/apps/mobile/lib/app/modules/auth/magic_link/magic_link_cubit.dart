import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/models/user.model.dart';
import '../../../data/repositories/auth.repository.dart';
import '../../../core/errors/app_exception.dart';

part 'magic_link_state.dart';

class MagicLinkCubit extends Cubit<MagicLinkState> {
  MagicLinkCubit({required AuthRepository authRepository})
      : _authRepository = authRepository,
        super(const MagicLinkInitial());

  final AuthRepository _authRepository;

  Future<void> sendLink({required String email}) async {
    if (state is MagicLinkLoading) return;
    emit(const MagicLinkLoading());
    try {
      await _authRepository.sendMagicLink(email: email);
      emit(MagicLinkSent(email));
    } on ApiException catch (e) {
      emit(MagicLinkFailure(e.message));
    } catch (e) {
      emit(MagicLinkFailure(e.toString()));
    }
  }

  Future<void> verifyLink({required String token}) async {
    if (token.isEmpty) return;
    if (state is MagicLinkLoading) return;
    emit(const MagicLinkLoading());
    try {
      final response = await _authRepository.verifyMagicLink(token: token);
      emit(MagicLinkVerified(token: response.token, user: response.user));
    } on ApiException catch (e) {
      emit(MagicLinkFailure(e.message));
    } catch (e) {
      emit(MagicLinkFailure(e.toString()));
    }
  }
}
