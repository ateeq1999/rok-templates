import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../core/utils/snackbar_helper.dart';
import '../../routes/app_routes.dart';
import 'org_detail_cubit.dart';

class OrgDetailView extends StatefulWidget {
  const OrgDetailView({super.key, required this.orgId});

  final String orgId;

  @override
  State<OrgDetailView> createState() => _OrgDetailViewState();
}

class _OrgDetailViewState extends State<OrgDetailView> {
  @override
  void initState() {
    super.initState();
    context.read<OrgDetailCubit>().load(widget.orgId);
  }

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<OrgDetailCubit, OrgDetailState>(
      listener: (context, state) {
        if (state is OrgDetailFailure) {
          SnackbarHelper.showError(context, state.message);
        }
      },
      builder: (context, state) {
        if (state is OrgDetailLoading) {
          return const Scaffold(
            body: Center(child: CircularProgressIndicator()),
          );
        }
        if (state is OrgDetailLoaded) {
          return Scaffold(
            appBar: AppBar(
              title: Text(state.org.name),
              actions: [
                IconButton(
                  icon: const Icon(Icons.edit),
                  onPressed: () => _showRenameDialog(context, state.org.name),
                ),
              ],
            ),
            body: ListView(
              children: [
                ListTile(
                  leading: const Icon(Icons.group),
                  title: Text('${state.members.length} Members'),
                  trailing: TextButton.icon(
                    icon: const Icon(Icons.person_add),
                    label: const Text('Invite'),
                    onPressed: () =>
                        context.push(AppRoutes.orgInvite(widget.orgId)),
                  ),
                ),
                const Divider(),
                ...state.members.map(
                  (m) => ListTile(
                    leading: CircleAvatar(
                      child: Text(
                          (m.name ?? m.email ?? m.userId)[0].toUpperCase()),
                    ),
                    title: Text(m.name ?? m.email ?? m.userId),
                    subtitle: m.email != null ? Text(m.email!) : null,
                    trailing: Chip(label: Text(m.role)),
                    onLongPress: () =>
                        _showMemberActions(context, m.userId, m.role),
                  ),
                ),
              ],
            ),
          );
        }
        return const Scaffold(body: SizedBox.shrink());
      },
    );
  }

  void _showRenameDialog(BuildContext context, String currentName) {
    final controller = TextEditingController(text: currentName);
    showDialog<void>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Rename Organization'),
        content: TextField(
          controller: controller,
          decoration: const InputDecoration(border: OutlineInputBorder()),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              final name = controller.text.trim();
              if (name.isNotEmpty && name != currentName) {
                Navigator.of(dialogContext).pop();
                context.read<OrgDetailCubit>().updateOrg(name);
              }
            },
            child: const Text('Rename'),
          ),
        ],
      ),
    );
  }

  void _showMemberActions(BuildContext context, String userId, String role) {
    showModalBottomSheet<void>(
      context: context,
      builder: (sheetContext) => Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          ListTile(
            leading: const Icon(Icons.admin_panel_settings),
            title: Text(
                role == 'admin' ? 'Remove Admin Role' : 'Make Admin'),
            onTap: () {
              Navigator.of(sheetContext).pop();
              context.read<OrgDetailCubit>().updateMemberRole(
                  userId, role == 'admin' ? 'member' : 'admin');
            },
          ),
          ListTile(
            leading: const Icon(Icons.person_remove, color: Colors.red),
            title: const Text('Remove Member',
                style: TextStyle(color: Colors.red)),
            onTap: () {
              Navigator.of(sheetContext).pop();
              context.read<OrgDetailCubit>().removeMember(userId);
            },
          ),
        ],
      ),
    );
  }
}
