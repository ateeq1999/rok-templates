import 'dart:typed_data';

import 'package:api_client/api_client.dart';

/// The signed-in user's profile, through the generated client's `users`
/// and `avatars` operations. Every failure is the client's [ApiException].
class UserRepository {
  UserRepository(this._api);

  final ApiClient _api;

  Future<UserResponse> getMe() => _api.users.me();

  /// Sets the display name; `null` clears it.
  Future<UserResponse> updateDisplayName(String? displayName) =>
      _api.users.updateMe(
        UpdateProfileDto(displayName: Nullable(displayName)),
      );

  /// Makes `bytes` (a JPEG, PNG or WebP image, its type `contentType`)
  /// the profile photo.
  Future<UserResponse> uploadAvatar(List<int> bytes, String contentType) =>
      _api.avatars.upload(
        Bytes(Uint8List.fromList(bytes), contentType: contentType),
      );

  Future<UserResponse> deleteAvatar() => _api.avatars.remove();

  /// The profile photo's URL on the API, or `null` without one: the API
  /// answers it as a path on its origin.
  Uri? avatarUri(UserResponse user) => switch (user.avatarUrl) {
        final String path when path.startsWith('/') => _api.url(path),
        final String url => Uri.tryParse(url),
        null => null,
      };
}
