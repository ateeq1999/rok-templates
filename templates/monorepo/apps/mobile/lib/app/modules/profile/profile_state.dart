part of 'profile_cubit.dart';

sealed class ProfileState {
  const ProfileState();
}

final class ProfileInitial extends ProfileState {
  const ProfileInitial();
}

final class ProfileLoading extends ProfileState {
  const ProfileLoading();
}

final class ProfileLoaded extends ProfileState {
  const ProfileLoaded(this.user);
  final UserResponse user;
}

final class ProfileFailure extends ProfileState {
  const ProfileFailure(this.message);
  final String message;
}

/// A new verification link is on its way.
final class ProfileVerificationSent extends ProfileState {
  const ProfileVerificationSent();
}
