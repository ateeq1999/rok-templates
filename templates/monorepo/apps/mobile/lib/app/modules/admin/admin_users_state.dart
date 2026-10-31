part of 'admin_users_cubit.dart';

sealed class AdminUsersState {
  const AdminUsersState();
}

final class AdminUsersInitial extends AdminUsersState {
  const AdminUsersInitial();
}

final class AdminUsersLoading extends AdminUsersState {
  const AdminUsersLoading();
}

final class AdminUsersLoaded extends AdminUsersState {
  const AdminUsersLoaded({
    required this.users,
    this.cursor,
    required this.hasMore,
    this.search = '',
  });

  final List<AdminUser> users;
  final String? cursor;
  final bool hasMore;
  final String search;

  AdminUsersLoaded copyWith({
    List<AdminUser>? users,
    String? cursor,
    bool? hasMore,
    String? search,
  }) {
    return AdminUsersLoaded(
      users: users ?? this.users,
      cursor: cursor,
      hasMore: hasMore ?? this.hasMore,
      search: search ?? this.search,
    );
  }
}

final class AdminUsersFailure extends AdminUsersState {
  const AdminUsersFailure(this.message);
  final String message;
}
