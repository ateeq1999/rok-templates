import 'dart:async';
import 'dart:convert';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';

import '../data/models/user.model.dart';

enum AuthStatus { unknown, authenticated, unauthenticated }

/// Manages token + user persistence and broadcasts [AuthStatus] changes.
///
/// BUG-02 fix: refreshToken logic is now handled in DioService interceptor.
/// BUG-04 fix: User is stored as JSON, not a URL query string.
///
/// DioService subscribes to [getToken] for header injection.
/// AuthBloc subscribes to [status] for reactive auth state.
class AuthService {
  static const _tokenKey = 'auth_token';
  static const _userKey = 'current_user';

  final _storage = const FlutterSecureStorage();
  final _statusController = StreamController<AuthStatus>.broadcast();

  /// Stream of auth status changes. AuthBloc subscribes to this.
  Stream<AuthStatus> get status async* {
    // Emit initial state based on stored token.
    final token = await _storage.read(key: _tokenKey);
    yield token != null ? AuthStatus.authenticated : AuthStatus.unauthenticated;
    yield* _statusController.stream;
  }

  Future<String?> getToken() => _storage.read(key: _tokenKey);

  Future<User?> getUser() async {
    final raw = await _storage.read(key: _userKey);
    if (raw == null) return null;
    try {
      return User.fromJson(jsonDecode(raw) as Map<String, dynamic>);
    } catch (_) {
      return null;
    }
  }

  Future<void> saveToken(String token) async {
    await _storage.write(key: _tokenKey, value: token);
    _statusController.add(AuthStatus.authenticated);
  }

  Future<void> saveUser(User user) async {
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
