import 'dart:convert';

import 'package:api_client/api_client.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:mobile/app/data/repositories/user.repository.dart';

import '../support/api.dart';

void main() {
  late Requests requests;

  setUp(() => requests = []);

  UserRepository repository(Map<String, Object?> answer) => UserRepository(
        fakeApi(
          (_) async => json(answer),
          session: Session(token: () => 'tok-1'),
          requests: requests,
        ),
      );

  test('getMe reads the signed-in user with the session token', () async {
    final user = await repository(userJson()).getMe();

    expect(requests.single.url.path, '/api/v1/users/me');
    expect(requests.single.headers['Authorization'], 'Bearer tok-1');
    expect(user.displayName, 'Ada');
    expect(user.role, Role.user);
  });

  test('updateDisplayName patches the display name', () async {
    final user = await repository(userJson(displayName: 'Ada L.'))
        .updateDisplayName('Ada L.');

    expect(requests.single.method, 'PATCH');
    expect(jsonDecode(requests.single.body), {'display_name': 'Ada L.'});
    expect(user.displayName, 'Ada L.');
  });

  test('uploadAvatar puts the image bytes with their type', () async {
    await repository(userJson(avatarUrl: '/api/v1/avatars/a.png'))
        .uploadAvatar([1, 2, 3], 'image/png');

    final request = requests.single;
    expect(request.method, 'PUT');
    expect(request.url.path, '/api/v1/users/me/avatar');
    expect(request.headers['Content-Type'], 'image/png');
    expect(request.bodyBytes, [1, 2, 3]);
  });

  test('avatarUri resolves the API\'s path on its origin', () {
    final repo = repository(userJson());
    final withAvatar =
        UserResponse.fromJson(userJson(avatarUrl: '/api/v1/avatars/a.png'));

    expect(repo.avatarUri(withAvatar).toString(),
        '$apiOrigin/api/v1/avatars/a.png');
    expect(repo.avatarUri(testUser()), isNull);
  });
}
