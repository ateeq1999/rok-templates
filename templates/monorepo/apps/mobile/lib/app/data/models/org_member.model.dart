class OrgMember {
  const OrgMember({
    required this.userId,
    this.name,
    this.email,
    required this.role,
    required this.joinedAt,
  });

  final String userId;
  final String? name;
  final String? email;
  final String role;
  final DateTime joinedAt;

  factory OrgMember.fromJson(Map<String, dynamic> json) {
    return OrgMember(
      userId: json['userId'] as String,
      name: json['name'] as String?,
      email: json['email'] as String?,
      role: json['role'] as String,
      joinedAt: DateTime.parse(json['joinedAt'] as String),
    );
  }

  OrgMember copyWith({String? role}) {
    return OrgMember(
      userId: userId,
      name: name,
      email: email,
      role: role ?? this.role,
      joinedAt: joinedAt,
    );
  }
}
