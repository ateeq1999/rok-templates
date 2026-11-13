import 'package:api_client/api_client.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repositories/auth.repository.dart';
import '../../../services/browser.dart';

part 'linked_identities_state.dart';

/// The settings' Linked identities: the account's, the providers it can
/// link, linking one in the system browser and unlinking one. Nothing to
/// show while no provider is configured.
class LinkedIdentitiesCubit extends Cubit<LinkedIdentitiesState> {
  LinkedIdentitiesCubit({
    required AuthRepository authRepository,
    required Browser browser,
  })  : _authRepository = authRepository,
        _browser = browser,
        super(const LinkedIdentitiesState());

  final AuthRepository _authRepository;
  final Browser _browser;

  /// Reads the configured providers and, when there are any, the account's
  /// identities.
  Future<void> load() async {
    try {
      final providers = await _authRepository.signInProviders();
      if (providers.isEmpty) {
        emit(const LinkedIdentitiesState());
        return;
      }
      final identities = await _authRepository.identities();
      emit(LinkedIdentitiesState(providers: providers, identities: identities));
    } on ApiException catch (e) {
      emit(state.copyWith(message: e.message));
    }
  }

  /// Opens linking an identity at `provider` in the system browser; it
  /// comes back by the deep link, which opens the settings again.
  Future<void> link(String provider) async {
    try {
      await _browser.open(await _authRepository.linkUrl(provider));
    } on ApiException catch (e) {
      emit(state.copyWith(message: e.message));
    }
  }

  /// Unlinks `identity`; the API refuses the account's last way in, saying
  /// why.
  Future<void> unlink(IdentityResponse identity) async {
    try {
      await _authRepository.unlink(identity.id);
      emit(state.copyWith(
        identities: [
          for (final kept in state.identities)
            if (kept.id != identity.id) kept,
        ],
      ));
    } on ApiException catch (e) {
      emit(state.copyWith(message: e.message));
    }
  }
}
