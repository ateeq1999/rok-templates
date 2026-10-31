import 'package:api_client/api_client.dart';

/// A form's view of the client's [ApiException]: one message per field the
/// API rejected (a `422`'s first issue for each), keyed by the field's name
/// on the wire (`email`, `new_password`...); `null` when the failure is
/// not about the fields.
extension FieldErrors on ApiException {
  Map<String, String>? get fieldErrors {
    final errors = {
      for (final MapEntry(:key, :value) in details.entries)
        if (value.isNotEmpty) key: value.first.message,
    };
    return errors.isEmpty ? null : errors;
  }
}
