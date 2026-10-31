import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../core/utils/snackbar_helper.dart';
import '../../data/models/admin_user.model.dart';
import 'admin_users_cubit.dart';

class AdminUserDetailView extends StatelessWidget {
  const AdminUserDetailView({super.key, required this.user});

  final AdminUser user;

  @override
  Widget build(BuildContext context) {
    return BlocListener<AdminUsersCubit, AdminUsersState>(
      listener: (context, state) {
        if (state is AdminUsersFailure) {
          SnackbarHelper.showError(context, state.message);
        }
      },
      child: Scaffold(
        appBar: AppBar(
          title: Text(user.name ?? user.email),
          actions: [
            IconButton(
              icon: const Icon(Icons.delete, color: Colors.red),
              tooltip: 'Delete user',
              onPressed: () => _confirmDelete(context),
            ),
          ],
        ),
        body: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            _InfoRow('ID', user.id),
            _InfoRow('Email', user.email),
            _InfoRow('Name', user.name ?? '—'),
            _InfoRow('Role', user.role ?? 'user'),
            _InfoRow(
                'Email Verified', user.emailVerified ? 'Yes' : 'No'),
            _InfoRow('Banned', user.banned ? 'Yes' : 'No'),
            _InfoRow('Deleted', user.deleted ? 'Yes' : 'No'),
            _InfoRow('Created', user.createdAt.toLocal().toString()),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              icon: Icon(
                  user.banned ? Icons.check_circle : Icons.block),
              label: Text(user.banned ? 'Unban User' : 'Ban User'),
              style: ElevatedButton.styleFrom(
                backgroundColor:
                    user.banned ? Colors.green : Colors.orange,
                foregroundColor: Colors.white,
              ),
              onPressed: () => context
                  .read<AdminUsersCubit>()
                  .updateUser(user.id, {'banned': !user.banned}),
            ),
            const SizedBox(height: 12),
            OutlinedButton.icon(
              icon: const Icon(Icons.admin_panel_settings),
              label: Text(user.role == 'admin'
                  ? 'Remove Admin Role'
                  : 'Make Admin'),
              onPressed: () => context
                  .read<AdminUsersCubit>()
                  .updateUser(user.id, {
                'role': user.role == 'admin' ? 'user' : 'admin',
              }),
            ),
          ],
        ),
      ),
    );
  }

  void _confirmDelete(BuildContext context) {
    showDialog<void>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Delete User'),
        content: Text(
            'Delete ${user.name ?? user.email}? This action cannot be undone.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            style:
                ElevatedButton.styleFrom(backgroundColor: Colors.red),
            onPressed: () {
              Navigator.of(dialogContext).pop();
              context.read<AdminUsersCubit>().deleteUser(user.id);
              Navigator.of(context).pop();
            },
            child: const Text('Delete',
                style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
  }
}

class _InfoRow extends StatelessWidget {
  const _InfoRow(this.label, this.value);

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 120,
            child: Text(label,
                style: const TextStyle(
                    fontWeight: FontWeight.w600, color: Colors.grey)),
          ),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }
}
