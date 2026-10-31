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

final class SignInSuccess extends SignInState {
  const SignInSuccess({this.token, required this.user});
  // null when email verification is required before first sign-in
  final String? token;
  final User user;
}

/// API indicated 2FA is required to complete sign-in.
final class SignInTwoFactorRequired extends SignInState {
  const SignInTwoFactorRequired();
}

final class SignInFailure extends SignInState {
  const SignInFailure(this.message, {this.fieldErrors});
  final String message;
  final Map<String, String>? fieldErrors;
}
