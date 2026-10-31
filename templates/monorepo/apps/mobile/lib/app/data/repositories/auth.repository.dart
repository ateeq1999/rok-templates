import 'package:api_client/api_client.dart';

import '../../services/auth_service.dart';

/// What signing in answered: a session, or the second step two-factor
/// sign-in asks for.
sealed class SignInResult {
  const SignInResult();
}

/// Signed in: the session is saved and [user] is who it belongs to.
final class SignedIn extends SignInResult {
  const SignedIn({required this.user, required this.token});
  final UserResponse user;
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
  AuthRepository(this._api, this._session);

  final ApiClient _api;
  final AuthService _session;

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
  Future<UserResponse> signUp({
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
}
