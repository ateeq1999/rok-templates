part of 'auth_bloc.dart';

sealed class AuthEvent {
  const AuthEvent();
}

/// Fired on app start to hydrate auth state from storage.
final class AuthStarted extends AuthEvent {
  const AuthStarted();
}

/// Fired after a successful sign-in or OAuth callback.
final class AuthUserChanged extends AuthEvent {
  const AuthUserChanged({required this.user, required this.token});
  final User user;
  final String token;
}

/// Fired when the user explicitly signs out or the token refresh fails.
final class AuthSignedOut extends AuthEvent {
  const AuthSignedOut();
}
