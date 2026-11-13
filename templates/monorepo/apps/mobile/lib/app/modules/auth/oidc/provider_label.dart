/// A provider's name as people read it: `google` is "Google", `acme-id`
/// "Acme Id". The API knows providers only by the name in `OIDC_PROVIDERS`.
String providerLabel(String name) => name
    .split(RegExp('[-_]'))
    .where((word) => word.isNotEmpty)
    .map((word) => word[0].toUpperCase() + word.substring(1))
    .join(' ');
