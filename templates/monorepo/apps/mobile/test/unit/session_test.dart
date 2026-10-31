import 'package:api_client/api_client.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:mobile/app/services/auth_service.dart';

import '../support/api.dart';

void main() {
  late AuthService session;

  setUp(() {
    FlutterSecureStorage.setMockInitialValues({});
    session = AuthService();
  });

  tearDown(() => session.dispose());

  test('a 401 to the session\'s token signs out', () async {
    await session.saveToken('tok-1');
    await session.saveUser(testUser());
    final api = fakeApi(
      (_) async => error(401, 'UNAUTHORIZED', 'the session has expired'),
      session: session,
    );
    final statuses = <AuthStatus>[];
    final subscription = session.status.listen(statuses.add);

    await expectLater(api.users.me(), throwsA(isA<ApiException>()));
    await pumpEventQueue();

    expect(await session.getToken(), isNull);
    expect(await session.getUser(), isNull);
    expect(statuses.last, AuthStatus.unauthenticated);
    await subscription.cancel();
  });

  test('a 401 to a token the session has replaced keeps the new one',
      () async {
    await session.saveToken('tok-2');

    await session.expired('tok-1');

    expect(await session.getToken(), 'tok-2');
  });

  test('the stored user survives a restart', () async {
    await session.saveUser(testUser(totpEnabled: true));

    final user = await AuthService().getUser();

    expect(user?.email, 'ada@example.com');
    expect(user?.totpEnabled, isTrue);
  });
}
