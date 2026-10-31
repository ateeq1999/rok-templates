import 'user.model.dart';

class AuthResponse {
  // token is null after sign-up (email verification required before sign-in)
  final String? token;
  final User user;

  const AuthResponse({this.token, required this.user});

  factory AuthResponse.fromJson(Map<String, dynamic> json) {
    return AuthResponse(
      token: json['token'] as String?,
      user: User.fromJson(json['user'] as Map<String, dynamic>),
    );
  }

  Map<String, dynamic> toJson() {
    return {'token': token, 'user': user.toJson()};
  }
}
