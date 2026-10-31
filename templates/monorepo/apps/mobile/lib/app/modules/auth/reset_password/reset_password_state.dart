part of 'reset_password_cubit.dart';

sealed class ResetPasswordState {
  const ResetPasswordState();
}

final class ResetPasswordInitial extends ResetPasswordState {
  const ResetPasswordInitial();
}

final class ResetPasswordLoading extends ResetPasswordState {
  const ResetPasswordLoading();
}

final class ResetPasswordSuccess extends ResetPasswordState {
  const ResetPasswordSuccess();
}

final class ResetPasswordFailure extends ResetPasswordState {
  const ResetPasswordFailure(this.message, {this.fieldErrors});
  final String message;
  final Map<String, String>? fieldErrors;
}
