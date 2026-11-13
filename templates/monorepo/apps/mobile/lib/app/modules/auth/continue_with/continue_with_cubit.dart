import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repositories/auth.repository.dart';
import '../../../services/browser.dart';

/// The "Continue with …" buttons: the providers the API is configured with
/// (none, and there are no buttons), and starting a sign-in with one in the
/// system browser. The browser comes back by the deep link the
/// [OidcCubit] finishes.
class ContinueWithCubit extends Cubit<List<String>> {
  ContinueWithCubit({
    required AuthRepository authRepository,
    required Browser browser,
  })  : _authRepository = authRepository,
        _browser = browser,
        super(const []);

  final AuthRepository _authRepository;
  final Browser _browser;

  /// Reads the configured providers. A failure leaves the buttons hidden:
  /// the password form still works.
  Future<void> load() async {
    try {
      emit(await _authRepository.signInProviders());
    } on Object {
      emit(const []);
    }
  }

  /// Opens `provider`'s sign-in in the system browser.
  Future<void> continueWith(String provider) =>
      _browser.open(_authRepository.signInStartUrl(provider));
}
