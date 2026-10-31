import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../core/utils/snackbar_helper.dart';
import '../../core/widgets/empty_state.dart';
import '../../routes/app_routes.dart';
import 'org_list_cubit.dart';

class OrgsListView extends StatefulWidget {
  const OrgsListView({super.key});

  @override
  State<OrgsListView> createState() => _OrgsListViewState();
}

class _OrgsListViewState extends State<OrgsListView> {
  @override
  void initState() {
    super.initState();
    context.read<OrgListCubit>().loadOrgs();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Organizations')),
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showCreateDialog(context),
        child: const Icon(Icons.add),
      ),
      body: BlocConsumer<OrgListCubit, OrgListState>(
        listener: (context, state) {
          if (state is OrgListFailure) {
            SnackbarHelper.showError(context, state.message);
          }
        },
        builder: (context, state) {
          if (state is OrgListLoading) {
            return const Center(child: CircularProgressIndicator());
          }
          if (state is OrgListLoaded) {
            return RefreshIndicator(
              onRefresh: () => context.read<OrgListCubit>().loadOrgs(),
              child: state.orgs.isEmpty
                  ? ListView(
                      physics: const AlwaysScrollableScrollPhysics(),
                      children: [
                        const SizedBox(height: 80),
                        EmptyState(
                          icon: Icons.business_outlined,
                          title: 'No organizations yet',
                          message: 'Create one to get started.',
                          actionLabel: 'Create',
                          onAction: () => _showCreateDialog(context),
                        ),
                      ],
                    )
                  : ListView.separated(
                      physics: const AlwaysScrollableScrollPhysics(),
                      itemCount: state.orgs.length,
                      separatorBuilder: (_, _) => const Divider(height: 1),
                      itemBuilder: (context, i) {
                        final org = state.orgs[i];
                        return ListTile(
                          leading:
                              const CircleAvatar(child: Icon(Icons.business)),
                          title: Text(org.name),
                          subtitle: Text('Created ${org.createdAt.toLocal()}'),
                          trailing: const Icon(Icons.chevron_right),
                          onTap: () =>
                              context.push(AppRoutes.orgDetail(org.id)),
                        );
                      },
                    ),
            );
          }
          return const SizedBox.shrink();
        },
      ),
    );
  }

  void _showCreateDialog(BuildContext context) {
    final nameController = TextEditingController();
    showDialog<void>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Create Organization'),
        content: TextField(
          controller: nameController,
          decoration: const InputDecoration(
            labelText: 'Organization Name',
            border: OutlineInputBorder(),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              final name = nameController.text.trim();
              if (name.isNotEmpty) {
                Navigator.of(dialogContext).pop();
                context.read<OrgListCubit>().createOrg(name);
              }
            },
            child: const Text('Create'),
          ),
        ],
      ),
    );
  }
}
