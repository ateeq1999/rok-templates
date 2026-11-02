// The Dart client's behaviour against the monorepo's contract, run by the
// `dart_client` gate (tests/gates/dart_client.rs)
// inside a scaffolded `clients/dart`, over a mock HTTP client: the typed
// calls put the right request on the wire, read the answer into the
// models, and fail with one `ApiException`.

import 'dart:convert';

import 'package:api_client/api_client.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:test/test.dart';

/// A client whose every request is answered by `answer`, recording them.
({ApiClient api, List<http.Request> sent}) client(
  Future<http.Response> Function(http.Request request) answer, {
  Session? session,
}) {
  final sent = <http.Request>[];
  final api = ApiClient(
    baseUrl: 'https://api.test/',
    session: session,
    httpClient: MockClient((request) {
      sent.add(request);
      return answer(request);
    }),
  );
  return (api: api, sent: sent);
}

http.Response json(int status, Object? body) => http.Response(
  jsonEncode(body),
  status,
  headers: {'content-type': 'application/json'},
);

const user = {
  'id': '7f1c',
  'email': 'ann@example.com',
  'display_name': null,
  'email_verified': true,
  'has_password': true,
  'is_active': true,
  'role': 'admin',
  'totp_enabled': false,
  'created_at': '2026-10-31T00:00:00Z',
};

void main() {
  test('a public call sends no token and reads a oneOf variant', () async {
    final c = client(
      (_) async => json(200, {
        'access_token': 'jwt',
        'expires_in': 900,
        'token_type': 'Bearer',
      }),
      session: Session(token: () => 'stale'),
    );
    final answer = await c.api.auth.login(
      const CredentialsDto(email: 'ann@example.com', password: 'secret12'),
    );
    final token = switch (answer) {
      TokenResponse(:final accessToken) => accessToken,
      LoginResponsePendingToken() => null,
    };
    expect(token, 'jwt');
    final request = c.sent.single;
    expect(request.method, 'POST');
    expect(request.url.toString(), 'https://api.test/api/v1/auth/login');
    expect(request.headers.containsKey('Authorization'), isFalse);
    expect(jsonDecode(request.body), {
      'email': 'ann@example.com',
      'password': 'secret12',
    });
  });

  test('a secured call sends the session token, the path encoded, and a '
      'cleared field as null', () async {
    final c = client(
      (_) async => json(200, user),
      session: Session(token: () => 'jwt'),
    );
    final updated = await c.api.users.update(
      'a/b',
      const UpdateUserDto(displayName: Nullable(null), role: Nullable(Role.user)),
    );
    expect(updated.role, Role.admin);
    expect(updated.displayName, isNull);
    final request = c.sent.single;
    expect(request.method, 'PATCH');
    expect(request.url.path, '/api/v1/users/a%2Fb');
    expect(request.headers['Authorization'], 'Bearer jwt');
    expect(jsonDecode(request.body), {'display_name': null, 'role': 'user'});
  });

  test('the query is sent by its wire names, unset ones left out', () async {
    final c = client(
      (_) async => json(200, {
        'items': [user],
        'page': 2,
        'per_page': 10,
        'total': 11,
      }),
    );
    final page = await c.api.users.list(
      page: 2,
      perPage: 10,
      sort: UserSort.email,
    );
    expect(page.items.single.email, 'ann@example.com');
    expect(page.total, 11);
    expect(c.sent.single.url.queryParameters, {
      'page': '2',
      'per_page': '10',
      'sort': 'email',
    });
  });

  test('a 422 surfaces the validation issues by field', () async {
    final c = client(
      (_) async => json(422, {
        'error': {
          'code': 'VALIDATION_FAILED',
          'message': 'The request is invalid',
          'details': {
            'email': [
              {'code': 'email', 'message': 'Not an email address'},
            ],
          },
        },
      }),
    );
    final error = await c.api.auth
        .register(const CredentialsDto(email: 'nope', password: 'secret12'))
        .then<ApiException?>((_) => null, onError: (Object e) => e as ApiException);
    expect(error!.status, 422);
    expect(error.code, 'VALIDATION_FAILED');
    expect(error.isValidation, isTrue);
    expect(error.issuesOf('email').single.message, 'Not an email address');
  });

  test('a 401 to a call that sent the token ends the session, unless the '
      'call keeps it', () async {
    final expired = <String>[];
    final c = client(
      (_) async => json(401, {
        'error': {'code': 'UNAUTHORIZED', 'message': 'Sign in again'},
      }),
      session: Session(token: () => 'jwt', onExpired: expired.add),
    );
    await expectLater(c.api.users.me(), throwsA(isA<ApiException>()));
    expect(expired, ['jwt']);
    await expectLater(
      c.api.users.me(options: const RequestOptions(keepSession: true)),
      throwsA(isA<ApiException>()),
    );
    expect(expired, ['jwt']);
  });

  test('a request that never reaches the API is status 0', () async {
    final c = client((_) async => throw http.ClientException('offline'));
    await expectLater(
      c.api.users.me(),
      throwsA(
        isA<ApiException>()
            .having((e) => e.status, 'status', 0)
            .having((e) => e.code, 'code', 'network_error'),
      ),
    );
  });
}
