abstract class AppRoutes {
  static const splash = '/splash';
  static const signIn = '/sign-in';
  static const signUp = '/sign-up';
  static const forgotPassword = '/forgot-password';
  static const resetPassword = '/reset-password';
  static const verifyEmail = '/verify-email';
  static const twoFactor = '/two-factor';
  /// Where a sign-in (or a link) with a provider comes back: the deep link
  /// `<APP_SCHEME>://auth/oidc?…`.
  static const oidc = '/auth/oidc';
  static const home = '/home';
  static const profile = '/profile';
  static const settings = '/settings';
  static const appearance = '/settings/appearance';
}
