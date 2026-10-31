class AuditLog {
  const AuditLog({
    required this.id,
    required this.userId,
    required this.action,
    this.metadata,
    required this.createdAt,
  });

  final String id;
  final String userId;
  final String action;
  final Map<String, dynamic>? metadata;
  final DateTime createdAt;

  factory AuditLog.fromJson(Map<String, dynamic> json) {
    return AuditLog(
      id: json['id'] as String,
      userId: json['userId'] as String,
      action: json['action'] as String,
      metadata: json['metadata'] as Map<String, dynamic>?,
      createdAt: DateTime.parse(json['createdAt'] as String),
    );
  }
}
