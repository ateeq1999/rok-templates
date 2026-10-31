class OrgInvitation {
  const OrgInvitation({
    required this.id,
    required this.email,
    required this.role,
    required this.token,
    required this.status,
    required this.createdAt,
  });

  final String id;
  final String email;
  final String role;
  final String token;
  final String status;
  final DateTime createdAt;

  factory OrgInvitation.fromJson(Map<String, dynamic> json) {
    return OrgInvitation(
      id: json['id'] as String,
      email: json['email'] as String,
      role: json['role'] as String,
      token: json['token'] as String,
      status: json['status'] as String,
      createdAt: DateTime.parse(json['createdAt'] as String),
    );
  }
}
