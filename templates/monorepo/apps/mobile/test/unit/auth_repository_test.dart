import 'dart:convert';

import 'package:api_client/api_client.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;

import 'package:mobile/app/core/errors/field_errors.dart';
import 'package:mobile/app/data/repositories/auth.repository.dart';
import 'package:mobile/app/services/auth_service.dart';

import '../support/api.dart';

void main() {
  late AuthService session;
  late Requests requests;

  setUp(() {
    FlutterSecureStorage.setMockInitialValues({});
    session = AuthService();
    requests = [];
  });

  tearDown(() => session.dispose());

  AuthRepository repository(
    Future<http.Response> Function(http.Request request) answer,
  ) =>
      AuthRepository(
        fakeApi(answer, session: session, requests: requests),
        session,
      );

  group('AuthRepository.signIn', () {
    test('a session answer is saved, sent to /auth/me, and signs in',
        () async {
      final repo = repository((request) async => switch (request.url.path) {
            '/api/v1/auth/login' => json({
                'access_token': 'tok-1',
                'expires_in': 900,
                'token_type': 'Bearer',
              }),
            '/api/v1/auth/me' => json(userJson()),
            _ => json(null, status: 404),
          });

      final result = await repo.signIn(email: 'ada@example.com', password: 'pw');

      expect(result, isA<SignedIn>());
      final signedIn = result as SignedIn;
      expect(signedIn.token, 'tok-1');
      expect(signedIn.user.email, 'ada@example.com');
      expect(await session.getToken(), 'tok-1');
      expect((await session.getUser())?.displayName, 'Ada');

      final login = requests.first;
      expect(login.method, 'POST');
      expect(jsonDecode(login.body),
          {'email': 'ada@example.com', 'password': 'pw'});
      expect(login.headers['Authorization'], isNull);
      expect(requests.last.url.path, '/api/v1/auth/me');
      expect(requests.last.headers['Authorization'], 'Bearer tok-1');
    });

    test('an account with two-factor on answers the second step', () async {
      final repo = repository((_) async => json({
            'pending_token': 'pending-1',
            'requires_totp': true,
          }));

      final result = await repo.signIn(email: 'ada@example.com', password: 'pw');

      expect(result, isA<TwoFactorRequired>());
      expect((result as TwoFactorRequired).pendingToken, 'pending-1');
      expect(await session.getToken(), isNull);
    });

    test('wrong credentials are the API\'s 401, and no session starts',
        () async {
      final repo = repository((_) async =>
          error(401, 'UNAUTHORIZED', 'invalid email or password'));

      await expectLater(
        repo.signIn(email: 'ada@example.com', password: 'nope'),
        throwsA(isA<ApiException>()
            .having((e) => e.status, 'status', 401)
            .having((e) => e.message, 'message', 'invalid email or password')),
      );
      expect(await session.getToken(), isNull);
    });

    test('a 422 names the fields it rejected', () async {
      final repo = repository((_) async => error(
            422,
            'VALIDATION_FAILED',
            'the request is invalid',
            details: {
              'email': [
                {'code': 'email', 'message': 'not an email address'},
              ],
            },
          ));

      await expectLater(
        repo.signIn(email: 'bad', password: 'pw'),
        throwsA(isA<ApiException>().having(
          (e) => e.fieldErrors,
          'fieldErrors',
          {'email': 'not an email address'},
        )),
      );
    });
  });

  test('verifyTwoFactor sends the code with the pending token and signs in',
      () async {
    final repo = repository((request) async => switch (request.url.path) {
          '/api/v1/auth/2fa/verify' => json({
              'access_token': 'tok-2',
              'expires_in': 900,
              'token_type': 'Bearer',
            }),
          _ => json(userJson(totpEnabled: true)),
        });

    final signedIn =
        await repo.verifyTwoFactor(pendingToken: 'pending-1', code: '123456');

    expect(jsonDecode(requests.first.body),
        {'code': '123456', 'pending_token': 'pending-1'});
    expect(signedIn.user.totpEnabled, isTrue);
    expect(await session.getToken(), 'tok-2');
  });

  test('changing the password ends the session (the API ends them all)',
      () async {
    await session.saveToken('tok-1');
    final repo =
        repository((_) async => json({'message': 'password changed'}));

    await repo.changePassword(currentPassword: 'old', newPassword: 'new-pw-1');

    expect(jsonDecode(requests.single.body),
        {'current_password': 'old', 'new_password': 'new-pw-1'});
    expect(await session.getToken(), isNull);
  });

  test('a wrong password to turn two-factor off keeps the session', () async {
    await session.saveToken('tok-1');
    final repo =
        repository((_) async => error(401, 'UNAUTHORIZED', 'wrong password'));

    await expectLater(
      repo.disableTwoFactor(currentPassword: 'nope'),
      throwsA(isA<ApiException>().having((e) => e.status, 'status', 401)),
    );
    expect(requests.single.headers['Authorization'], 'Bearer tok-1');
    expect(await session.getToken(), 'tok-1');
  });

  test('signing up registers an email and a password', () async {
    final repo = repository((_) async =>
        json(userJson(emailVerified: false), status: 201));

    final user = await repo.signUp(email: 'ada@example.com', password: 'pw-12345');

    expect(requests.single.url.path, '/api/v1/auth/register');
    expect(user.emailVerified, isFalse);
  });
}
