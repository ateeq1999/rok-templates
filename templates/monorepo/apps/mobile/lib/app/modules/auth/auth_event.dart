part of 'auth_bloc.dart';

sealed class AuthEvent {
  const AuthEvent();
}

/// Fired on app start to hydrate auth state from storage.
final class AuthStarted extends AuthEvent {
  const AuthStarted();
}

/// Fired after a successful sign-in, or when the signed-in user changes.
final class AuthUserChanged extends AuthEvent {
  const AuthUserChanged({required this.user, required this.token});
  final AccountResponse user;
  final String token;
}

/// Fired when the user signs out or the API rejects the session.
final class AuthSignedOut extends AuthEvent {
  const AuthSignedOut();
}
