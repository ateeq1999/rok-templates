import 'package:api_client/api_client.dart';

import '../../services/auth_service.dart';
import '../oidc_start_url.dart';

/// What signing in answered: a session, or the second step two-factor
/// sign-in asks for.
sealed class SignInResult {
  const SignInResult();
}

/// Signed in: the session is saved and [user] is who it belongs to.
final class SignedIn extends SignInResult {
  const SignedIn({required this.user, required this.token});
  final AccountResponse user;
  final String token;
}

/// The account has two-factor on: [pendingToken] goes with the code to
/// [AuthRepository.verifyTwoFactor].
final class TwoFactorRequired extends SignInResult {
  const TwoFactorRequired(this.pendingToken);
  final String pendingToken;
}

/// The account flows, through the generated client's `auth` operations.
/// Every failure is the client's [ApiException].
class AuthRepository {
  /// `apiUrl` is the API's origin, which the system browser opens a
  /// sign-in with a provider at.
  AuthRepository(this._api, this._session, {required String apiUrl})
      : _apiUrl = apiUrl;

  final ApiClient _api;
  final AuthService _session;
  final String _apiUrl;

  /// The two-factor calls are secured, but their `401` is a wrong code or
  /// password, not an expired session.
  static const _keepSession = RequestOptions(keepSession: true);

  Future<SignInResult> signIn({
    required String email,
    required String password,
  }) async {
    final answer = await _api.auth.login(
      CredentialsDto(email: email, password: password),
    );
    return switch (answer) {
      TokenResponse(:final accessToken) => _startSession(accessToken),
      LoginResponsePendingToken(:final pendingToken) =>
        TwoFactorRequired(pendingToken),
    };
  }

  /// A two-factor sign-in's second step: the authenticator's `code`.
  Future<SignedIn> verifyTwoFactor({
    required String pendingToken,
    required String code,
  }) async {
    final session = await _api.auth.verify2fa(
      VerifyTotpDto(code: code, pendingToken: pendingToken),
    );
    return _startSession(session.accessToken);
  }

  /// Saves `token` as the session, then reads whose it is.
  Future<SignedIn> _startSession(String token) async {
    await _session.saveToken(token);
    try {
      final user = await _api.auth.me();
      await _session.saveUser(user);
      return SignedIn(user: user, token: token);
    } on Object {
      await _session.signOut();
      rethrow;
    }
  }

  /// A new account; the API mails a link to verify its email before it
  /// signs in.
  Future<NewAccountResponse> signUp({
    required String email,
    required String password,
  }) =>
      _api.auth.register(CredentialsDto(email: email, password: password));

  /// The API has no sign-out call: the token is forgotten here.
  Future<void> signOut() => _session.signOut();

  Future<void> forgotPassword({required String email}) =>
      _api.auth.forgot(ForgotPasswordDto(email: email));

  Future<void> resetPassword({
    required String token,
    required String password,
  }) =>
      _api.auth.reset(ResetPasswordDto(newPassword: password, token: token));

  /// Changes the password. The API signs every session out with it, this
  /// one too.
  Future<void> changePassword({
    required String currentPassword,
    required String newPassword,
  }) async {
    await _api.auth.change(
      ChangePasswordDto(
        currentPassword: currentPassword,
        newPassword: newPassword,
      ),
    );
    await _session.signOut();
  }

  Future<void> verifyEmail({required String token}) =>
      _api.auth.verifyEmail(VerifyEmailDto(token: token));

  Future<void> sendVerificationEmail() => _api.auth.resend();

  /// A new authenticator secret, not enforced until [enableTwoFactor].
  Future<TotpSetupResponse> setUpTwoFactor() =>
      _api.auth.setup(options: _keepSession);

  /// Turns two-factor on with the first `code`; the recovery codes.
  Future<List<String>> enableTwoFactor({required String code}) async {
    final enabled = await _api.auth.enable(
      EnableTotpDto(code: code),
      options: _keepSession,
    );
    return enabled.recoveryCodes;
  }

  Future<void> disableTwoFactor({required String currentPassword}) =>
      _api.auth.disable(
        DisableTotpDto(currentPassword: currentPassword),
        options: _keepSession,
      );

  /// The providers people may sign in with, by name (such as `google`):
  /// none, and the App shows no "Continue with …" button.
  Future<List<String>> signInProviders() async {
    final providers = await _api.auth.listSignInProviders();
    return [for (final provider in providers) provider.name];
  }

  /// Where the system browser starts signing in with `provider`; the
  /// provider ends at the deep link [signInWithHandoff] takes the code of.
  Uri signInStartUrl(String provider) =>
      oidcStartUrl(apiUrl: _apiUrl, provider: provider);

  /// Finishes a sign-in with a provider: trades the one-minute `code` of
  /// the deep link `<APP_SCHEME>://auth/oidc?code=…` for a session.
  Future<SignedIn> signInWithHandoff(String code) async {
    final session = await _api.auth.exchangeHandoff(HandoffDto(code: code));
    return _startSession(session.accessToken);
  }

  /// The signed-in account's Linked identities, oldest first.
  Future<List<IdentityResponse>> identities() => _api.auth.listIdentities();

  /// Where the system browser goes to link an identity at `provider` to
  /// the signed-in account; it comes back by `<APP_SCHEME>://auth/oidc?linked=…`.
  Future<Uri> linkUrl(String provider) async {
    final link = await _api.auth.linkIdentity(
      provider,
      client: OidcClient.mobile,
    );
    return Uri.parse(link.url);
  }

  /// Unlinks the identity `id`. The API refuses the account's last way in
  /// (a `422` saying why).
  Future<void> unlink(String id) => _api.auth.unlinkIdentity(id);
}
