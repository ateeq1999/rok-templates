import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:mobile/app/core/errors/app_exception.dart';

DioException _dioError({
  int? statusCode,
  dynamic data,
  DioExceptionType type = DioExceptionType.badResponse,
}) {
  final req = RequestOptions(path: '/v1/api/test');
  return DioException(
    requestOptions: req,
    type: type,
    response: statusCode == null
        ? null
        : Response<dynamic>(
            requestOptions: req,
            statusCode: statusCode,
            data: data,
          ),
  );
}

void main() {
  group('ApiException.fromDioError — envelope error shape', () {
    test('parses {success:false, error:{code,message}}', () {
      final ex = ApiException.fromDioError(_dioError(
        statusCode: 404,
        data: {
          'success': false,
          'error': {'code': 'NOT_FOUND', 'message': 'User not found'},
          'meta': {'timestamp': 'now', 'requestId': 'r1'},
        },
      ));

      expect(ex.statusCode, 404);
      expect(ex.message, 'User not found');
      expect(ex.code, 'NOT_FOUND');
      expect(ex.fieldErrors, isNull);
    });

    test('parses validation details as string list', () {
      final ex = ApiException.fromDioError(_dioError(
        statusCode: 400,
        data: {
          'success': false,
          'error': {
            'code': 'VALIDATION_ERROR',
            'message': 'Invalid input',
            'details': ['title must not be empty', 'slug required'],
          },
        },
      ));

      expect(ex.code, 'VALIDATION_ERROR');
      expect(ex.details, ['title must not be empty', 'slug required']);
      expect(ex.fieldErrors, isNull);
    });

    test('parses validation details as [{path, message}] → fieldErrors', () {
      final ex = ApiException.fromDioError(_dioError(
        statusCode: 400,
        data: {
          'success': false,
          'error': {
            'code': 'VALIDATION_ERROR',
            'message': 'Invalid input',
            'details': [
              {'path': 'email', 'message': 'Must be a valid email'},
              {'path': 'password', 'message': 'Too short'},
            ],
          },
        },
      ));

      expect(ex.fieldErrors, {
        'email': 'Must be a valid email',
        'password': 'Too short',
      });
    });

    test('accepts [{field, error}] alternate keys', () {
      final ex = ApiException.fromDioError(_dioError(
        statusCode: 400,
        data: {
          'success': false,
          'error': {
            'code': 'VALIDATION_ERROR',
            'message': 'Bad',
            'details': [
              {'field': 'name', 'error': 'Required'},
            ],
          },
        },
      ));

      expect(ex.fieldErrors, {'name': 'Required'});
    });
  });

  group('ApiException.fromDioError — auth-style direct shape', () {
    test('parses {message} body', () {
      final ex = ApiException.fromDioError(_dioError(
        statusCode: 401,
        data: {'message': 'Invalid credentials'},
      ));

      expect(ex.statusCode, 401);
      expect(ex.message, 'Invalid credentials');
      expect(ex.code, isNull);
      expect(ex.fieldErrors, isNull);
    });

    test('parses {message, fieldErrors:{email:"..."}}', () {
      final ex = ApiException.fromDioError(_dioError(
        statusCode: 400,
        data: {
          'message': 'Invalid',
          'fieldErrors': {'email': 'Already registered'},
        },
      ));

      expect(ex.fieldErrors, {'email': 'Already registered'});
    });
  });

  group('ApiException.fromDioError — network / timeout', () {
    test('connection timeout message', () {
      final ex = ApiException.fromDioError(
        _dioError(type: DioExceptionType.connectionTimeout),
      );
      expect(ex.statusCode, isNull);
      expect(ex.message, contains('timed out'));
    });

    test('no-response falls back to no-network message', () {
      final ex = ApiException.fromDioError(
        _dioError(type: DioExceptionType.connectionError),
      );
      expect(ex.message, contains('No internet'));
    });
  });

  group('ApiException.fromDioError — malformed bodies', () {
    test('non-map body → generic server error', () {
      final ex = ApiException.fromDioError(_dioError(
        statusCode: 500,
        data: '<html>500</html>',
      ));
      expect(ex.statusCode, 500);
      expect(ex.message, 'Server error');
    });

    test('envelope without error field → generic', () {
      final ex = ApiException.fromDioError(_dioError(
        statusCode: 500,
        data: {'success': false},
      ));
      expect(ex.message, 'Server error');
    });
  });
}
