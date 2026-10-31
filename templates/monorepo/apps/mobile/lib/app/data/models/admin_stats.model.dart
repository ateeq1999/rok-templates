class AdminStats {
  const AdminStats({
    required this.total,
    required this.admins,
    required this.banned,
    required this.deleted,
  });

  final int total;
  final int admins;
  final int banned;
  final int deleted;

  factory AdminStats.fromJson(Map<String, dynamic> json) {
    return AdminStats(
      total: json['total'] as int? ?? 0,
      admins: json['admins'] as int? ?? 0,
      banned: json['banned'] as int? ?? 0,
      deleted: json['deleted'] as int? ?? 0,
    );
  }
}
