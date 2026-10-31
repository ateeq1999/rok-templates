import 'package:api_client/api_client.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:mobile/app/core/errors/field_errors.dart';

void main() {
  test('fieldErrors takes each rejected field\'s first message', () {
    const exception = ApiException(
      422,
      'VALIDATION_FAILED',
      'the request is invalid',
      details: {
        'new_password': [
          FieldIssue(code: 'length', message: 'at least 8 characters'),
          FieldIssue(code: 'breached', message: 'found in a breach'),
        ],
        'token': [],
      },
    );

    expect(exception.fieldErrors, {'new_password': 'at least 8 characters'});
  });

  test('fieldErrors is null when no field was rejected', () {
    const exception = ApiException(0, 'network_error', 'Cannot reach the server');

    expect(exception.fieldErrors, isNull);
  });
}
