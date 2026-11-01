import 'dart:async';
import 'dart:convert';

import 'package:api_client/api_client.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

enum AuthStatus { unknown, authenticated, unauthenticated }

/// The session: the API's bearer token and the signed-in user, kept in
/// secure storage, and the [AuthStatus] changes the [AuthBloc] follows.
///
/// It is the generated client's [Session]: the client reads [token] before
/// every secured call, and calls [expired] when the API rejects the token
/// (a `401`), which signs out. The API has no refresh token: an expired
/// session signs in again.
class AuthService implements Session {
  AuthService({FlutterSecureStorage? storage})
      : _storage = storage ?? const FlutterSecureStorage();

  static const _tokenKey = 'auth_token';
  static const _userKey = 'current_user';

  final FlutterSecureStorage _storage;
  final _statusController = StreamController<AuthStatus>.broadcast();

  /// Stream of auth status changes. AuthBloc subscribes to this.
  Stream<AuthStatus> get status async* {
    // Emit initial state based on stored token.
    final token = await _storage.read(key: _tokenKey);
    yield token != null ? AuthStatus.authenticated : AuthStatus.unauthenticated;
    yield* _statusController.stream;
  }

  Future<String?> getToken() => _storage.read(key: _tokenKey);

  @override
  Future<String?> token() => getToken();

  /// The API rejected `token`: when it is still this session's, the
  /// session is over.
  @override
  Future<void> expired(String token) async {
    if (await getToken() == token) await signOut();
  }

  Future<AccountResponse?> getUser() async {
    final raw = await _storage.read(key: _userKey);
    if (raw == null) return null;
    try {
      return AccountResponse.fromJson(jsonDecode(raw) as Map<String, Object?>);
    } on Object {
      return null;
    }
  }

  Future<void> saveToken(String token) async {
    await _storage.write(key: _tokenKey, value: token);
    _statusController.add(AuthStatus.authenticated);
  }

  Future<void> saveUser(AccountResponse user) async {
    await _storage.write(key: _userKey, value: jsonEncode(user.toJson()));
  }

  Future<void> signOut() async {
    await _storage.delete(key: _tokenKey);
    await _storage.delete(key: _userKey);
    _statusController.add(AuthStatus.unauthenticated);
  }

  void dispose() {
    _statusController.close();
  }
}
