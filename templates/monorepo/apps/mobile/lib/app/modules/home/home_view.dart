import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../core/config/feature_flags.dart';
import '../../routes/app_routes.dart';
import '../auth/auth_bloc.dart';

class HomeView extends StatelessWidget {
  const HomeView({super.key});

  @override
  Widget build(BuildContext context) {
    final flags = context.read<FeatureFlags>();
    return BlocBuilder<AuthBloc, AuthState>(
      builder: (context, state) {
        final user =
            state is AuthAuthenticated ? state.user : null;

        return Scaffold(
          appBar: AppBar(title: const Text('Home')),
          body: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Welcome, ${user?.name ?? 'User'}!',
                  style: Theme.of(context).textTheme.headlineMedium,
                ),
                const SizedBox(height: 16),
                Text('Email: ${user?.email ?? ''}'),
                const SizedBox(height: 32),
                ListTile(
                  leading: const Icon(Icons.person),
                  title: const Text('Profile'),
                  onTap: () => context.push(AppRoutes.profile),
                ),
                ListTile(
                  leading: const Icon(Icons.settings),
                  title: const Text('Settings'),
                  onTap: () => context.push(AppRoutes.settings),
                ),
                if (flags.organizations)
                  ListTile(
                    leading: const Icon(Icons.business),
                    title: const Text('Organizations'),
                    onTap: () => context.push(AppRoutes.organizations),
                  ),
                if (flags.admin && user?.role == 'admin')
                  ListTile(
                    leading: const Icon(Icons.admin_panel_settings,
                        color: Colors.deepPurple),
                    title: const Text('Admin Dashboard'),
                    onTap: () => context.push(AppRoutes.admin),
                  ),
              ],
            ),
          ),
        );
      },
    );
  }
}
