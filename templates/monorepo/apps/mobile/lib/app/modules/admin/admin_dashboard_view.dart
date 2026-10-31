import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../routes/app_routes.dart';
import 'admin_stats_cubit.dart';

class AdminDashboardView extends StatefulWidget {
  const AdminDashboardView({super.key});

  @override
  State<AdminDashboardView> createState() => _AdminDashboardViewState();
}

class _AdminDashboardViewState extends State<AdminDashboardView> {
  @override
  void initState() {
    super.initState();
    context.read<AdminStatsCubit>().loadStats();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Admin Dashboard')),
      body: BlocBuilder<AdminStatsCubit, AdminStatsState>(
        builder: (context, state) {
          if (state is AdminStatsLoading) {
            return const Center(child: CircularProgressIndicator());
          }
          if (state is AdminStatsFailure) {
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Text(state.message),
                  const SizedBox(height: 16),
                  ElevatedButton(
                    onPressed: context.read<AdminStatsCubit>().loadStats,
                    child: const Text('Retry'),
                  ),
                ],
              ),
            );
          }

          final stats =
              state is AdminStatsLoaded ? state.stats : null;

          return ListView(
            padding: const EdgeInsets.all(16),
            children: [
              if (stats != null) ...[
                _StatsGrid(stats: [
                  ('Total Users', stats.total, Icons.people),
                  ('Admins', stats.admins, Icons.admin_panel_settings),
                  ('Banned', stats.banned, Icons.block),
                  ('Deleted', stats.deleted, Icons.delete),
                ]),
                const SizedBox(height: 24),
              ],
              ListTile(
                leading: const Icon(Icons.manage_accounts),
                title: const Text('Manage Users'),
                trailing: const Icon(Icons.chevron_right),
                onTap: () => context.push(AppRoutes.adminUsers),
              ),
              const Divider(),
              ListTile(
                leading: const Icon(Icons.history),
                title: const Text('Audit Logs'),
                trailing: const Icon(Icons.chevron_right),
                onTap: () => context.push(AppRoutes.adminAuditLogs),
              ),
            ],
          );
        },
      ),
    );
  }
}

class _StatsGrid extends StatelessWidget {
  const _StatsGrid({required this.stats});

  final List<(String label, int value, IconData icon)> stats;

  @override
  Widget build(BuildContext context) {
    return GridView.count(
      crossAxisCount: 2,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      crossAxisSpacing: 12,
      mainAxisSpacing: 12,
      childAspectRatio: 1.6,
      children: stats
          .map((s) => Card(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(s.$3, size: 28),
                      const SizedBox(height: 8),
                      Text('${s.$2}',
                          style: const TextStyle(
                              fontSize: 22, fontWeight: FontWeight.bold)),
                      Text(s.$1,
                          style: const TextStyle(color: Colors.grey),
                          textAlign: TextAlign.center),
                    ],
                  ),
                ),
              ))
          .toList(),
    );
  }
}
