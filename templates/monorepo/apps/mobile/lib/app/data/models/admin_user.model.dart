class AdminUser {
  const AdminUser({
    required this.id,
    required this.email,
    this.name,
    this.image,
    this.role,
    required this.emailVerified,
    required this.banned,
    required this.deleted,
    required this.createdAt,
  });

  final String id;
  final String email;
  final String? name;
  final String? image;
  final String? role;
  final bool emailVerified;
  final bool banned;
  final bool deleted;
  final DateTime createdAt;

  factory AdminUser.fromJson(Map<String, dynamic> json) {
    return AdminUser(
      id: json['id'] as String,
      email: json['email'] as String,
      name: json['name'] as String?,
      image: json['image'] as String?,
      role: json['role'] as String?,
      emailVerified: json['emailVerified'] as bool? ?? false,
      banned: json['banned'] as bool? ?? false,
      deleted: json['deleted'] as bool? ?? false,
      createdAt: DateTime.parse(json['createdAt'] as String),
    );
  }

  AdminUser copyWith({
    String? role,
    bool? banned,
    bool? emailVerified,
  }) {
    return AdminUser(
      id: id,
      email: email,
      name: name,
      image: image,
      role: role ?? this.role,
      emailVerified: emailVerified ?? this.emailVerified,
      banned: banned ?? this.banned,
      deleted: deleted,
      createdAt: createdAt,
    );
  }
}
