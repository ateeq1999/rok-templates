import '../../routes/app_routes.dart';
import '../config/app_config.dart';

/// Where in the App a deep link (`<APP_SCHEME>://auth/…`) goes, or none for
/// a link it doesn't answer:
///
/// - `auth/verify-email?token=…` and `auth/reset-password?token=…`, the
///   links the API mails, open the screens that redeem the token;
/// - `auth/oidc?…`, where the API's callback hands a sign-in (or a link)
///   with a provider back, opens the screen that finishes it.
String? deepLinkLocation(Uri uri) {
  if (uri.scheme != AppConfig.appScheme || uri.host != 'auth') return null;
  final query = uri.queryParameters;
  return switch (uri.path) {
    '/verify-email' when query['token'] != null =>
      Uri(path: AppRoutes.verifyEmail, queryParameters: {'token': query['token']})
          .toString(),
    '/reset-password' when query['token'] != null =>
      Uri(
        path: AppRoutes.resetPassword,
        queryParameters: {'token': query['token']},
      ).toString(),
    '/oidc' => Uri(path: AppRoutes.oidc, queryParameters: query).toString(),
    _ => null,
  };
}
