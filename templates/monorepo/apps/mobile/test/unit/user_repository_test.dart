import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:mobile/app/core/errors/app_exception.dart';
import 'package:mobile/app/data/providers/user.provider.dart';
import 'package:mobile/app/data/repositories/user.repository.dart';

class MockUserProvider extends Mock implements UserProvider {}

Response<dynamic> _response(dynamic data, {int status = 200}) {
  return Response<dynamic>(
    requestOptions: RequestOptions(path: '/v1/api/users/me'),
    statusCode: status,
    data: data,
  );
}

Map<String, dynamic> _envelope(Map<String, dynamic> data) => {
      'success': true,
      'data': data,
      'meta': {
        'timestamp': '2024-01-01T00:00:00.000Z',
        'requestId': 'req-1',
      },
    };

Map<String, dynamic> _userPayload() => {
      'id': 'u-1',
      'email': 'a@b.com',
      'name': 'Alice',
      'emailVerified': true,
      'twoFactorEnabled': true,
      'role': 'admin',
      'createdAt': '2024-01-01T00:00:00.000Z',
      'updatedAt': '2024-02-01T00:00:00.000Z',
    };

void main() {
  late MockUserProvider provider;
  late UserRepository repo;

  setUp(() {
    provider = MockUserProvider();
    repo = UserRepository(provider);
  });

  group('UserRepository.getMe', () {
    test('unwraps envelope and parses user fields', () async {
      when(() => provider.getMe())
          .thenAnswer((_) async => _response(_envelope(_userPayload())));

      final user = await repo.getMe();

      expect(user.id, 'u-1');
      expect(user.email, 'a@b.com');
      expect(user.twoFactorEnabled, isTrue);
      expect(user.role, 'admin');
      expect(user.updatedAt, isNotNull);
    });

    test('throws ApiException from envelope error', () async {
      final req = RequestOptions(path: '/v1/api/users/me');
      when(() => provider.getMe()).thenThrow(DioException(
        requestOptions: req,
        type: DioExceptionType.badResponse,
        response: Response<dynamic>(
          requestOptions: req,
          statusCode: 401,
          data: {
            'success': false,
            'error': {'code': 'UNAUTHORIZED', 'message': 'Token expired'},
          },
        ),
      ));

      expect(
        () => repo.getMe(),
        throwsA(isA<ApiException>()
            .having((e) => e.statusCode, 'statusCode', 401)
            .having((e) => e.code, 'code', 'UNAUTHORIZED')),
      );
    });
  });

  group('UserRepository.updateProfile', () {
    test('unwraps envelope on patch response', () async {
      when(() => provider.updateProfile(
            name: any(named: 'name'),
            email: any(named: 'email'),
          )).thenAnswer(
        (_) async => _response(
          _envelope({..._userPayload(), 'name': 'Alice Updated'}),
        ),
      );

      final user = await repo.updateProfile(name: 'Alice Updated');

      expect(user.name, 'Alice Updated');
    });
  });
}
