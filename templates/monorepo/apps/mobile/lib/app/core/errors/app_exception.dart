class ApiException implements Exception {
  final int? statusCode;
  final String message;
  final String? code;
  final Map<String, String>? fieldErrors;
  final List<String>? details;

  ApiException({
    this.statusCode,
    required this.message,
    this.code,
    this.fieldErrors,
    this.details,
  });

  factory ApiException.fromDioError(dynamic error) {
    final response = error.response;
    if (response != null) {
      final data = response.data;
      final statusCode = response.statusCode as int?;
      if (data is Map) {
        return _parseBody(data, statusCode);
      }
      return ApiException(statusCode: statusCode, message: 'Server error');
    }
    final type = error.type?.toString() ?? '';
    if (type.contains('connectionTimeout') ||
        type.contains('sendTimeout') ||
        type.contains('receiveTimeout')) {
      return ApiException(message: 'Request timed out. Please try again.');
    }
    return ApiException(message: 'No internet connection. Please check your network.');
  }

  static ApiException _parseBody(Map data, int? statusCode) {
    if (data['success'] == false && data['error'] is Map) {
      final err = data['error'] as Map;
      final msg = (err['message'] as String?) ?? 'Request failed';
      final code = err['code'] as String?;
      final parsed = _parseDetails(err['details']);
      return ApiException(
        statusCode: statusCode,
        message: msg,
        code: code,
        fieldErrors: parsed.fields,
        details: parsed.messages,
      );
    }
    if (data.containsKey('message')) {
      final parsed = _parseDetails(data['fieldErrors'] ?? data['details']);
      return ApiException(
        statusCode: statusCode,
        message: (data['message'] as String?) ?? 'An error occurred',
        code: data['code'] as String?,
        fieldErrors: parsed.fields,
        details: parsed.messages,
      );
    }
    return ApiException(statusCode: statusCode, message: 'Server error');
  }

  static _DetailsParseResult _parseDetails(dynamic raw) {
    if (raw == null) return const _DetailsParseResult(null, null);
    if (raw is Map) {
      final fields = <String, String>{};
      raw.forEach((k, v) {
        if (v is String) fields[k.toString()] = v;
        if (v is List && v.isNotEmpty) fields[k.toString()] = v.first.toString();
      });
      return _DetailsParseResult(fields.isEmpty ? null : fields, null);
    }
    if (raw is List) {
      final fields = <String, String>{};
      final messages = <String>[];
      for (final item in raw) {
        if (item is Map) {
          final path = item['path'] ?? item['field'] ?? item['property'];
          final msg = item['message'] ?? item['error'];
          if (path != null && msg != null) {
            fields[path.toString()] = msg.toString();
            continue;
          }
          if (msg != null) messages.add(msg.toString());
        } else if (item is String) {
          messages.add(item);
        }
      }
      return _DetailsParseResult(
        fields.isEmpty ? null : fields,
        messages.isEmpty ? null : messages,
      );
    }
    return const _DetailsParseResult(null, null);
  }
}

class _DetailsParseResult {
  final Map<String, String>? fields;
  final List<String>? messages;
  const _DetailsParseResult(this.fields, this.messages);
}

class AuthException implements Exception {
  final String message;
  final int? statusCode;

  AuthException({required this.message, this.statusCode});
}
