import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:mobile/app/core/errors/app_exception.dart';
import 'package:mobile/app/data/providers/auth.provider.dart';
import 'package:mobile/app/data/repositories/auth.repository.dart';

class MockAuthProvider extends Mock implements AuthProvider {}

Response<dynamic> _response(dynamic data, {int status = 200}) {
  return Response<dynamic>(
    requestOptions: RequestOptions(path: '/v1/api/auth/sign-in'),
    statusCode: status,
    data: data,
  );
}

DioException _dioError({int? statusCode, dynamic data}) {
  final req = RequestOptions(path: '/v1/api/auth/sign-in');
  return DioException(
    requestOptions: req,
    type: DioExceptionType.badResponse,
    response: Response<dynamic>(
      requestOptions: req,
      statusCode: statusCode,
      data: data,
    ),
  );
}

void main() {
  late MockAuthProvider provider;
  late AuthRepository repo;

  setUp(() {
    provider = MockAuthProvider();
    repo = AuthRepository(provider);
  });

  group('AuthRepository.signIn', () {
    test('returns AuthResponse with token on success', () async {
      when(() => provider.signIn(
            email: any(named: 'email'),
            password: any(named: 'password'),
          )).thenAnswer((_) async => _response({
            'token': 'bat_abc123',
            'user': {
              'id': 'u-1',
              'email': 'a@b.com',
              'name': 'Alice',
              'emailVerified': true,
              'createdAt': '2024-01-01T00:00:00.000Z',
            },
          }));

      final result =
          await repo.signIn(email: 'a@b.com', password: 'secret123');

      expect(result.token, 'bat_abc123');
      expect(result.user.email, 'a@b.com');
      expect(result.user.emailVerified, isTrue);
    });

    test('parses response when token is null (2FA-required / unverified)',
        () async {
      when(() => provider.signIn(
            email: any(named: 'email'),
            password: any(named: 'password'),
          )).thenAnswer((_) async => _response({
            'token': null,
            'user': {
              'id': 'u-2',
              'email': 'b@c.com',
              'name': 'Bob',
              'emailVerified': false,
              'twoFactorEnabled': true,
              'createdAt': '2024-01-01T00:00:00.000Z',
            },
          }));

      final result = await repo.signIn(email: 'b@c.com', password: 'x');

      expect(result.token, isNull);
      expect(result.user.twoFactorEnabled, isTrue);
    });

    test('throws ApiException with fieldErrors on envelope validation error',
        () async {
      when(() => provider.signIn(
            email: any(named: 'email'),
            password: any(named: 'password'),
          )).thenThrow(_dioError(statusCode: 400, data: {
        'success': false,
        'error': {
          'code': 'VALIDATION_ERROR',
          'message': 'Validation failed',
          'details': [
            {'path': 'email', 'message': 'Must be a valid email'},
          ],
        },
      }));

      expect(
        () => repo.signIn(email: 'bad', password: 'x'),
        throwsA(isA<ApiException>()
            .having((e) => e.statusCode, 'statusCode', 400)
            .having((e) => e.code, 'code', 'VALIDATION_ERROR')
            .having((e) => e.fieldErrors?['email'], 'fieldErrors.email',
                'Must be a valid email')),
      );
    });

    test('throws ApiException on Better-Auth-style 401', () async {
      when(() => provider.signIn(
            email: any(named: 'email'),
            password: any(named: 'password'),
          )).thenThrow(_dioError(
              statusCode: 401, data: {'message': 'Invalid credentials'}));

      expect(
        () => repo.signIn(email: 'a@b.com', password: 'nope'),
        throwsA(isA<ApiException>()
            .having((e) => e.statusCode, 'statusCode', 401)
            .having((e) => e.message, 'message', 'Invalid credentials')),
      );
    });
  });

  group('AuthRepository.signUp', () {
    test('returns AuthResponse with null token (email verification)', () async {
      when(() => provider.signUp(
            email: any(named: 'email'),
            password: any(named: 'password'),
            name: any(named: 'name'),
          )).thenAnswer((_) async => _response({
            'token': null,
            'user': {
              'id': 'u-3',
              'email': 'new@x.com',
              'name': 'New',
              'emailVerified': false,
              'createdAt': '2024-01-01T00:00:00.000Z',
            },
            'session': null,
          }));

      final r = await repo.signUp(
          email: 'new@x.com', password: 'pass12345', name: 'New');

      expect(r.token, isNull);
      expect(r.user.emailVerified, isFalse);
    });
  });

  group('AuthRepository.enableTwoFactor', () {
    test('returns uri + qrCode from API response', () async {
      when(() => provider.enableTwoFactor()).thenAnswer(
        (_) async => _response({
          'uri': 'otpauth://totp/app:user?secret=ABC',
          'qrCode': 'data:image/png;base64,iVBOR...',
        }),
      );

      final r = await repo.enableTwoFactor();

      expect(r.uri, startsWith('otpauth://'));
      expect(r.qrCode, startsWith('data:image/png'));
    });
  });
}
