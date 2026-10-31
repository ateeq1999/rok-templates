import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../core/utils/snackbar_helper.dart';
import '../../core/widgets/empty_state.dart';
import '../../routes/app_routes.dart';
import 'admin_users_cubit.dart';

class AdminUsersView extends StatefulWidget {
  const AdminUsersView({super.key});

  @override
  State<AdminUsersView> createState() => _AdminUsersViewState();
}

class _AdminUsersViewState extends State<AdminUsersView> {
  final _searchController = TextEditingController();

  @override
  void initState() {
    super.initState();
    context.read<AdminUsersCubit>().loadUsers();
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Users'),
        bottom: PreferredSize(
          preferredSize: const Size.fromHeight(56),
          child: Padding(
            padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
            child: TextField(
              controller: _searchController,
              decoration: InputDecoration(
                hintText: 'Search by name or email',
                prefixIcon: const Icon(Icons.search),
                suffixIcon: IconButton(
                  icon: const Icon(Icons.clear),
                  onPressed: () {
                    _searchController.clear();
                    context.read<AdminUsersCubit>().loadUsers();
                  },
                ),
                border: const OutlineInputBorder(),
                contentPadding: EdgeInsets.zero,
              ),
              onSubmitted: (q) =>
                  context.read<AdminUsersCubit>().loadUsers(search: q),
            ),
          ),
        ),
      ),
      body: BlocConsumer<AdminUsersCubit, AdminUsersState>(
        listener: (context, state) {
          if (state is AdminUsersFailure) {
            SnackbarHelper.showError(context, state.message);
          }
        },
        builder: (context, state) {
          if (state is AdminUsersLoading) {
            return const Center(child: CircularProgressIndicator());
          }
          if (state is AdminUsersLoaded) {
            return RefreshIndicator(
              onRefresh: () => context
                  .read<AdminUsersCubit>()
                  .loadUsers(search: _searchController.text),
              child: state.users.isEmpty
                  ? ListView(
                      physics: const AlwaysScrollableScrollPhysics(),
                      children: const [
                        SizedBox(height: 80),
                        EmptyState(
                          icon: Icons.person_search_outlined,
                          title: 'No users found',
                          message: 'Try a different search, or pull to refresh.',
                        ),
                      ],
                    )
                  : NotificationListener<ScrollNotification>(
                      onNotification: (n) {
                        if (n.metrics.pixels >=
                            n.metrics.maxScrollExtent - 200) {
                          context.read<AdminUsersCubit>().loadMore();
                        }
                        return false;
                      },
                      child: ListView.separated(
                        physics: const AlwaysScrollableScrollPhysics(),
                        itemCount:
                            state.users.length + (state.hasMore ? 1 : 0),
                        separatorBuilder: (_, _) => const Divider(height: 1),
                        itemBuilder: (context, i) {
                          if (i == state.users.length) {
                            return const Padding(
                              padding: EdgeInsets.all(16),
                              child: Center(child: CircularProgressIndicator()),
                            );
                          }
                          final user = state.users[i];
                          return ListTile(
                            leading: CircleAvatar(
                              backgroundImage: user.image != null
                                  ? NetworkImage(user.image!)
                                  : null,
                              child: user.image == null
                                  ? Text(
                                      (user.name ?? user.email)[0].toUpperCase())
                                  : null,
                            ),
                            title: Text(user.name ?? user.email),
                            subtitle: Text(user.email),
                            trailing: Row(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                if (user.banned)
                                  const Icon(Icons.block,
                                      color: Colors.red, size: 16),
                                if (user.role != null)
                                  Chip(
                                    label: Text(user.role!,
                                        style: const TextStyle(fontSize: 11)),
                                    padding: EdgeInsets.zero,
                                  ),
                                const Icon(Icons.chevron_right),
                              ],
                            ),
                            onTap: () => context
                                .push(AppRoutes.adminUserDetail(user.id)),
                          );
                        },
                      ),
                    ),
            );
          }
          return const SizedBox.shrink();
        },
      ),
    );
  }
}
