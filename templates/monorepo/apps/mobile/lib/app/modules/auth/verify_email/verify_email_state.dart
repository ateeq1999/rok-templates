part of 'verify_email_cubit.dart';

sealed class VerifyEmailState {
  const VerifyEmailState();
}

final class VerifyEmailInitial extends VerifyEmailState {
  const VerifyEmailInitial();
}

final class VerifyEmailLoading extends VerifyEmailState {
  const VerifyEmailLoading();
}

final class VerifyEmailSuccess extends VerifyEmailState {
  const VerifyEmailSuccess();
}

final class VerifyEmailFailure extends VerifyEmailState {
  const VerifyEmailFailure(this.message);
  final String message;
}
