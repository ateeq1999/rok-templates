part of 'sign_in_cubit.dart';

sealed class SignInState {
  const SignInState();
}

final class SignInInitial extends SignInState {
  const SignInInitial();
}

final class SignInLoading extends SignInState {
  const SignInLoading();
}

/// Signed in: the session is saved.
final class SignInSuccess extends SignInState {
  const SignInSuccess({required this.token, required this.user});
  final String token;
  final UserResponse user;
}

/// The account has two-factor on: the code goes with [pendingToken].
final class SignInTwoFactorRequired extends SignInState {
  const SignInTwoFactorRequired(this.pendingToken);
  final String pendingToken;
}

final class SignInFailure extends SignInState {
  const SignInFailure(this.message, {this.fieldErrors});
  final String message;
  final Map<String, String>? fieldErrors;
}
