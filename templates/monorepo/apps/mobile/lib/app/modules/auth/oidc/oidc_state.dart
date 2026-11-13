part of 'oidc_cubit.dart';

sealed class OidcState {
  const OidcState();
}

/// Reading the deep link, or trading its code for a session.
final class OidcFinishing extends OidcState {
  const OidcFinishing();
}

/// Signed in: the session is saved.
final class OidcSignedIn extends OidcState {
  const OidcSignedIn({required this.user, required this.token});
  final AccountResponse user;
  final String token;
}

/// The account has two-factor on: the code goes with [pendingToken].
final class OidcTwoFactorRequired extends OidcState {
  const OidcTwoFactorRequired(this.pendingToken);
  final String pendingToken;
}

/// An identity at [provider] was linked to the signed-in account.
final class OidcLinked extends OidcState {
  const OidcLinked(this.provider);
  final String provider;
}

/// Not signed in (or not linked), and why.
final class OidcRefused extends OidcState {
  const OidcRefused(this.message);
  final String message;
}
