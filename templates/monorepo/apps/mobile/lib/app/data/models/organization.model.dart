class Organization {
  const Organization({
    required this.id,
    required this.name,
    required this.createdAt,
  });

  final String id;
  final String name;
  final DateTime createdAt;

  factory Organization.fromJson(Map<String, dynamic> json) {
    return Organization(
      id: json['id'] as String,
      name: json['name'] as String,
      createdAt: DateTime.parse(json['createdAt'] as String),
    );
  }

  Organization copyWith({String? name}) {
    return Organization(id: id, name: name ?? this.name, createdAt: createdAt);
  }
}
