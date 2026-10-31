/// Unwraps the API response envelope `{success, data, meta}`.
///
/// Non-auth endpoints return:
///   `{"success": true, "data": {...}, "meta": {...}}`
/// Auth endpoints return direct data (no envelope).
///
/// Throws [FormatException] if `success` is false or `data` is absent.
Map<String, dynamic> unwrapEnvelope(dynamic responseData) {
  if (responseData is! Map<String, dynamic>) {
    throw const FormatException('Expected JSON object');
  }
  if (responseData.containsKey('success')) {
    if (responseData['success'] != true) {
      final err = responseData['error'];
      final msg = err is Map ? err['message'] ?? 'Request failed' : 'Request failed';
      throw FormatException(msg as String);
    }
    final data = responseData['data'];
    if (data is Map<String, dynamic>) return data;
    throw const FormatException('Missing data field in envelope');
  }
  return responseData;
}

/// Same as [unwrapEnvelope] but for list responses (`"data": [...]`).
List<Map<String, dynamic>> unwrapEnvelopeList(dynamic responseData) {
  if (responseData is! Map<String, dynamic>) {
    throw const FormatException('Expected JSON object');
  }
  if (responseData.containsKey('success')) {
    final data = responseData['data'];
    if (data is List) {
      return data.cast<Map<String, dynamic>>();
    }
    throw const FormatException('Expected list in data field');
  }
  if (responseData['data'] is List) {
    return (responseData['data'] as List).cast<Map<String, dynamic>>();
  }
  throw const FormatException('Expected list response');
}
