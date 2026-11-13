/// Where the system browser goes to sign in with `provider` from this App:
/// the API's `GET /api/v1/auth/oidc/{provider}/start?client=mobile`, which
/// redirects to the provider and, after its callback, opens the App's deep
/// link (`<APP_SCHEME>://auth/oidc?code=…`).
///
/// `start` answers only a redirect, so the released `rok` leaves it out of
/// the contract and `package:api_client`. After the next rok release, the
/// client generates a URL builder for it (`oidcStartUrl`): use that and
/// delete this file.
Uri oidcStartUrl({required String apiUrl, required String provider}) {
  final origin = apiUrl.endsWith('/')
      ? apiUrl.substring(0, apiUrl.length - 1)
      : apiUrl;
  return Uri.parse(
    '$origin/api/v1/auth/oidc/${Uri.encodeComponent(provider)}/start',
  ).replace(queryParameters: {'client': 'mobile'});
}
