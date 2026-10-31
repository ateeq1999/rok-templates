part of 'sign_up_cubit.dart';

sealed class SignUpState {
  const SignUpState();
}

final class SignUpInitial extends SignUpState {
  const SignUpInitial();
}

final class SignUpLoading extends SignUpState {
  const SignUpLoading();
}

final class SignUpSuccess extends SignUpState {
  const SignUpSuccess();
}

final class SignUpFailure extends SignUpState {
  const SignUpFailure(this.message, {this.fieldErrors});
  final String message;
  final Map<String, String>? fieldErrors;
}
