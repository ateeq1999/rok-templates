import 'dart:async';

import 'package:api_client/api_client.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../services/auth_service.dart';

part 'auth_event.dart';
part 'auth_state.dart';

/// Root BLoC that owns authentication state for the entire app.
///
/// • Subscribes to [AuthService.status] so any change (sign-in, sign-out,
///   401 refresh failure) automatically propagates to the router redirect.
/// • GoRouter listens to this bloc via [refreshListenable] to re-evaluate
///   the redirect guard on every state change.
class AuthBloc extends Bloc<AuthEvent, AuthState> {
  AuthBloc({required AuthService authService})
      : _authService = authService,
        super(const AuthInitial()) {
    on<AuthStarted>(_onStarted);
    on<AuthUserChanged>(_onUserChanged);
    on<AuthSignedOut>(_onSignedOut);

    _statusSubscription = authService.status.listen(
      (status) {
        if (status == AuthStatus.unauthenticated) {
          add(const AuthSignedOut());
        }
      },
    );
  }

  final AuthService _authService;
  late final StreamSubscription<AuthStatus> _statusSubscription;

  Future<void> _onStarted(AuthStarted event, Emitter<AuthState> emit) async {
    emit(const AuthLoading());
    try {
      final token = await _authService.getToken();
      if (token == null) {
        emit(const AuthUnauthenticated());
        return;
      }
      final user = await _authService.getUser();
      if (user == null) {
        emit(const AuthUnauthenticated());
        return;
      }
      emit(AuthAuthenticated(user: user, token: token));
    } catch (_) {
      emit(const AuthUnauthenticated());
    }
  }

  /// The signed-in user changed (signed in, or their profile did): kept
  /// with the session, so a restart shows it.
  Future<void> _onUserChanged(
    AuthUserChanged event,
    Emitter<AuthState> emit,
  ) async {
    await _authService.saveUser(event.user);
    emit(AuthAuthenticated(user: event.user, token: event.token));
  }

  void _onSignedOut(AuthSignedOut event, Emitter<AuthState> emit) {
    emit(const AuthUnauthenticated());
  }

  @override
  Future<void> close() {
    _statusSubscription.cancel();
    return super.close();
  }
}
