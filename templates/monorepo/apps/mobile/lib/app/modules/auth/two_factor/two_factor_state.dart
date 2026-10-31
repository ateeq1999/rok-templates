part of 'two_factor_cubit.dart';

sealed class TwoFactorState {
  const TwoFactorState();
}

final class TwoFactorInitial extends TwoFactorState {
  const TwoFactorInitial();
}

final class TwoFactorLoading extends TwoFactorState {
  const TwoFactorLoading();
}

final class TwoFactorSuccess extends TwoFactorState {
  const TwoFactorSuccess({this.token, required this.user});
  final String? token;
  final User user;
}

final class TwoFactorFailure extends TwoFactorState {
  const TwoFactorFailure(this.message);
  final String message;
}
