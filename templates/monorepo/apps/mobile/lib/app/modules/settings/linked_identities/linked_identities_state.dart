part of 'linked_identities_cubit.dart';

final class LinkedIdentitiesState {
  const LinkedIdentitiesState({
    this.providers = const [],
    this.identities = const [],
    this.message,
  });

  /// The configured providers, by name: none, and the section is hidden.
  final List<String> providers;

  /// The account's Linked identities, oldest first.
  final List<IdentityResponse> identities;

  /// Why the last call failed, such as an unlink of the last way in.
  final String? message;

  /// The providers the account has no identity at yet.
  List<String> get linkable => [
        for (final provider in providers)
          if (!identities.any((identity) => identity.provider == provider))
            provider,
      ];

  LinkedIdentitiesState copyWith({
    List<IdentityResponse>? identities,
    String? message,
  }) =>
      LinkedIdentitiesState(
        providers: providers,
        identities: identities ?? this.identities,
        message: message,
      );
}
