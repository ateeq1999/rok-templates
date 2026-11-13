import 'package:api_client/api_client.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repositories/auth.repository.dart';

part 'oidc_state.dart';

/// Finishes a sign-in (or a link) with a provider from the deep link the
/// API's callback opens, `<APP_SCHEME>://auth/oidc?…`: a hand-over `code`
/// traded for a session, a two-factor step, a linked identity, or why not.
class OidcCubit extends Cubit<OidcState> {
  OidcCubit({required AuthRepository authRepository})
      : _authRepository = authRepository,
        super(const OidcFinishing());

  final AuthRepository _authRepository;

  /// Reads the deep link's query `parameters`.
  Future<void> finish(Map<String, String> parameters) async {
    final code = parameters['code'];
    final pendingToken = parameters['pending_token'];
    final provider = parameters['linked'];
    if (code != null) {
      try {
        final signedIn = await _authRepository.signInWithHandoff(code);
        emit(OidcSignedIn(user: signedIn.user, token: signedIn.token));
      } on ApiException catch (e) {
        emit(OidcRefused(e.message));
      } catch (e) {
        emit(OidcRefused(e.toString()));
      }
    } else if (parameters['requires_totp'] == 'true' && pendingToken != null) {
      emit(OidcTwoFactorRequired(pendingToken));
    } else if (provider != null) {
      emit(OidcLinked(provider));
    } else {
      emit(OidcRefused(
        parameters['message'] ?? "The sign-in didn't finish. Please try again.",
      ));
    }
  }
}
