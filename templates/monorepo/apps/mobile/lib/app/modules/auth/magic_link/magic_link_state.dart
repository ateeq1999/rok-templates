part of 'magic_link_cubit.dart';

sealed class MagicLinkState {
  const MagicLinkState();
}

final class MagicLinkInitial extends MagicLinkState {
  const MagicLinkInitial();
}

final class MagicLinkLoading extends MagicLinkState {
  const MagicLinkLoading();
}

final class MagicLinkSent extends MagicLinkState {
  const MagicLinkSent(this.email);
  final String email;
}

final class MagicLinkVerified extends MagicLinkState {
  const MagicLinkVerified({this.token, required this.user});
  final String? token;
  final User user;
}

final class MagicLinkFailure extends MagicLinkState {
  const MagicLinkFailure(this.message);
  final String message;
}
