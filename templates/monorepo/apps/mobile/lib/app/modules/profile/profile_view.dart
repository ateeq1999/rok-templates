import 'package:cached_network_image/cached_network_image.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../core/utils/snackbar_helper.dart';
import 'profile_cubit.dart';

class ProfileView extends StatelessWidget {
  const ProfileView({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<ProfileCubit, ProfileState>(
      listener: (context, state) {
        if (state is ProfileFailure) {
          SnackbarHelper.showError(context, state.message);
        }
        if (state is ProfileLoaded) {
          // Dismiss any transient success messages here if needed.
        }
      },
      builder: (context, state) {
        final cubit = context.read<ProfileCubit>();
        final isLoading = state is ProfileLoading;
        final user = state is ProfileLoaded ? state.user : null;

        return Scaffold(
          appBar: AppBar(title: const Text('Profile')),
          body: isLoading
              ? const Center(child: CircularProgressIndicator())
              : Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    children: [
                      // BUG-06 fix: display actual avatar image when available.
                      GestureDetector(
                        onTap: cubit.pickAndUploadAvatar,
                        child: CircleAvatar(
                          radius: 50,
                          backgroundImage: user?.image != null
                              ? CachedNetworkImageProvider(user!.image!)
                              : null,
                          child: user?.image == null
                              ? const Icon(Icons.person, size: 50)
                              : null,
                        ),
                      ),
                      const SizedBox(height: 8),
                      TextButton(
                        onPressed: cubit.pickAndUploadAvatar,
                        child: const Text('Change Avatar'),
                      ),
                      const SizedBox(height: 24),
                      ListTile(
                        title: const Text('Name'),
                        subtitle: Text(user?.name ?? ''),
                        trailing: const Icon(Icons.edit),
                        onTap: () => _showEditNameDialog(context, cubit,
                            user?.name ?? ''),
                      ),
                      ListTile(
                        title: const Text('Email'),
                        subtitle: Text(user?.email ?? ''),
                        trailing: (user?.emailVerified ?? false)
                            ? const Icon(Icons.verified,
                                color: Colors.green, size: 20)
                            : null,
                      ),
                      if (user != null) ...[
                        const SizedBox(height: 8),
                        Wrap(
                          spacing: 8,
                          runSpacing: 8,
                          alignment: WrapAlignment.center,
                          children: [
                            if (user.twoFactorEnabled)
                              Chip(
                                avatar: const Icon(Icons.shield_outlined,
                                    size: 16),
                                label: const Text('2FA Enabled'),
                                backgroundColor: Theme.of(context)
                                    .colorScheme
                                    .primaryContainer,
                              )
                            else
                              const Chip(
                                avatar: Icon(Icons.shield_outlined, size: 16),
                                label: Text('2FA Disabled'),
                              ),
                            if (user.role == 'admin')
                              Chip(
                                avatar: const Icon(
                                    Icons.admin_panel_settings_outlined,
                                    size: 16),
                                label: const Text('Admin'),
                                backgroundColor: Theme.of(context)
                                    .colorScheme
                                    .tertiaryContainer,
                              ),
                          ],
                        ),
                      ],
                    ],
                  ),
                ),
        );
      },
    );
  }

  void _showEditNameDialog(
      BuildContext context, ProfileCubit cubit, String currentName) {
    final textController = TextEditingController(text: currentName);
    showDialog<void>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Edit Name'),
        content: TextFormField(
          controller: textController,
          decoration: const InputDecoration(labelText: 'Name'),
          // BUG-13 fix: validate non-empty
          validator: (v) =>
              (v == null || v.trim().isEmpty) ? 'Name cannot be empty' : null,
          autovalidateMode: AutovalidateMode.onUserInteraction,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              final name = textController.text.trim();
              if (name.isNotEmpty) {
                Navigator.of(dialogContext).pop();
                cubit.updateName(name);
              }
            },
            child: const Text('Save'),
          ),
        ],
      ),
    );
  }
}
