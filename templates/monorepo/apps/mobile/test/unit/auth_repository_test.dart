import 'package:flutter_test/flutter_test.dart';
import 'package:mobile/app/data/models/auth_response.model.dart';
import 'package:mobile/app/data/models/user.model.dart';

void main() {
  group('AuthResponse model', () {
    test('fromJson creates correct AuthResponse', () {
      final json = {
        'token': 'jwt-token-123',
        'user': {
          'id': 'user-123',
          'email': 'test@example.com',
          'name': 'Test User',
          'image': null,
          'emailVerified': true,
          'createdAt': '2024-01-01T00:00:00.000',
        },
      };

      final authResponse = AuthResponse.fromJson(json);

      expect(authResponse.token, equals('jwt-token-123'));
      expect(authResponse.user, isA<User>());
      expect(authResponse.user.email, equals('test@example.com'));
      expect(authResponse.user.name, equals('Test User'));
    });

    test('toJson creates correct JSON', () {
      final user = User(
        id: 'user-123',
        email: 'test@example.com',
        name: 'Test User',
        image: null,
        emailVerified: true,
        createdAt: DateTime(2024),
      );

      final authResponse = AuthResponse(token: 'jwt-token-123', user: user);
      final json = authResponse.toJson();

      expect(json['token'], equals('jwt-token-123'));
      expect(json['user'], isA<Map<String, dynamic>>());
    });
  });

  group('User model', () {
    test('fromJson creates correct User', () {
      final json = {
        'id': 'user-123',
        'email': 'test@example.com',
        'name': 'Test User',
        'image': 'https://example.com/avatar.jpg',
        'emailVerified': true,
        'createdAt': '2024-01-01T00:00:00.000',
      };

      final user = User.fromJson(json);

      expect(user.id, equals('user-123'));
      expect(user.email, equals('test@example.com'));
      expect(user.name, equals('Test User'));
      expect(user.image, equals('https://example.com/avatar.jpg'));
      expect(user.emailVerified, isTrue);
      expect(user.createdAt, equals(DateTime(2024)));
    });

    test('toJson serialises all fields', () {
      final user = User(
        id: 'user-123',
        email: 'test@example.com',
        name: 'Test User',
        image: null,
        emailVerified: false,
        createdAt: DateTime(2024),
      );

      final json = user.toJson();

      expect(json['id'], equals('user-123'));
      expect(json['email'], equals('test@example.com'));
      expect(json['name'], equals('Test User'));
      expect(json['image'], isNull);
      expect(json['emailVerified'], isFalse);
    });

    test('copyWith creates new User with updated fields', () {
      final user = User(
        id: 'user-123',
        email: 'test@example.com',
        name: 'Original Name',
        image: null,
        emailVerified: false,
        createdAt: DateTime(2024),
      );

      final updated = user.copyWith(name: 'Updated Name');

      expect(updated.id, equals('user-123'));
      expect(updated.name, equals('Updated Name'));
      expect(updated.email, equals('test@example.com'));
    });

    test('copyWith preserves fields not specified', () {
      final user = User(
        id: 'user-123',
        email: 'test@example.com',
        name: 'Name',
        image: 'https://example.com/img.jpg',
        emailVerified: true,
        createdAt: DateTime(2024),
      );

      final updated = user.copyWith(email: 'new@example.com');

      expect(updated.name, equals('Name'));
      expect(updated.image, equals('https://example.com/img.jpg'));
      expect(updated.emailVerified, isTrue);
    });
  });
}
