import 'dart:convert';

import 'package:api_client/api_client.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

/// The API's origin in tests; nothing listens there.
const apiOrigin = 'http://api.test';

/// A user as `apps/api` answers one (`UserResponse`).
Map<String, Object?> userJson({
  String id = 'u-1',
  String email = 'ada@example.com',
  String? displayName = 'Ada',
  String? avatarUrl,
  bool totpEnabled = false,
  bool emailVerified = true,
}) =>
    {
      'id': id,
      'email': email,
      'display_name': displayName,
      'avatar_url': avatarUrl,
      'email_verified': emailVerified,
      'has_password': true,
      'is_active': true,
      'role': 'user',
      'totp_enabled': totpEnabled,
      'created_at': '2026-01-01T00:00:00Z',
    };

/// The user [userJson] describes, as the client reads it.
UserResponse testUser({bool totpEnabled = false}) =>
    UserResponse.fromJson(userJson(totpEnabled: totpEnabled));

/// A JSON answer of `status`.
http.Response json(Object? body, {int status = 200}) => http.Response(
      jsonEncode(body),
      status,
      headers: {'content-type': 'application/json'},
    );

/// The API's error body (`ErrorBody`).
http.Response error(
  int status,
  String code,
  String message, {
  Map<String, Object?>? details,
}) =>
    json({
      'error': {
        'code': code,
        'message': message,
        'details': ?details,
      },
    }, status: status);

/// The requests a [fakeApi] received, in order.
typedef Requests = List<http.Request>;

/// A client of an API that answers with `answer`, recording each request
/// in `requests`.
ApiClient fakeApi(
  Future<http.Response> Function(http.Request request) answer, {
  Session? session,
  Requests? requests,
}) =>
    ApiClient(
      baseUrl: apiOrigin,
      session: session,
      httpClient: MockClient((request) {
        requests?.add(request);
        return answer(request);
      }),
    );
