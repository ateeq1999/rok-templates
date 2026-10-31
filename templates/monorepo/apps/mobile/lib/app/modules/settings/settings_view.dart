import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../core/config/feature_flags.dart';
import '../../core/utils/snackbar_helper.dart';
import '../../routes/app_routes.dart';
import '../auth/auth_bloc.dart';
import 'settings_cubit.dart';

class SettingsView extends StatelessWidget {
  const SettingsView({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<SettingsCubit, SettingsState>(
      listener: (context, state) {
        if (state is SettingsPasswordChanged) {
          SnackbarHelper.showSuccess(context, 'Password changed successfully');
          context.read<SettingsCubit>().resetToInitial();
        } else if (state is SettingsTwoFactorEnabled) {
          _showTwoFactorQrDialog(context, state.uri, state.qrCode);
        } else if (state is SettingsTwoFactorVerified) {
          SnackbarHelper.showSuccess(
              context, 'Two-factor authentication enabled');
          context.read<SettingsCubit>().resetToInitial();
        } else if (state is SettingsTwoFactorDisabled) {
          SnackbarHelper.showSuccess(
              context, 'Two-factor authentication disabled');
          context.read<SettingsCubit>().resetToInitial();
        } else if (state is SettingsAccountDeleted) {
          SnackbarHelper.showSuccess(context, 'Account deleted');
        } else if (state is SettingsFailure) {
          SnackbarHelper.showError(context, state.message);
          context.read<SettingsCubit>().resetToInitial();
        }
      },
      builder: (context, state) {
        final cubit = context.read<SettingsCubit>();
        final isLoading = state is SettingsLoading;
        final flags = context.read<FeatureFlags>();
        // FL7.6: read twoFactorEnabled from AuthBloc user
        final authState = context.watch<AuthBloc>().state;
        final twoFactorEnabled = authState is AuthAuthenticated
            ? authState.user.twoFactorEnabled
            : false;

        return Scaffold(
          appBar: AppBar(title: const Text('Settings')),
          body: isLoading
              ? const Center(child: CircularProgressIndicator())
              : ListView(
                  children: [
                    ListTile(
                      leading: const Icon(Icons.lock),
                      title: const Text('Change Password'),
                      trailing: const Icon(Icons.chevron_right),
                      onTap: () => _showChangePasswordDialog(context, cubit),
                    ),
                    const Divider(),
                    if (flags.twoFactor && !twoFactorEnabled)
                      ListTile(
                        leading: const Icon(Icons.security),
                        title: const Text('Enable Two-Factor Auth'),
                        subtitle: const Text('Add TOTP authenticator app'),
                        trailing: const Icon(Icons.chevron_right),
                        onTap: cubit.enableTwoFactor,
                      ),
                    if (flags.twoFactor && twoFactorEnabled)
                      ListTile(
                        leading: const Icon(Icons.verified_user,
                            color: Colors.green),
                        title: const Text('Two-Factor Auth'),
                        subtitle: const Text('Enabled'),
                        trailing: TextButton(
                          onPressed: () =>
                              _showDisableTwoFactorDialog(context, cubit),
                          child: const Text('Disable',
                              style: TextStyle(color: Colors.red)),
                        ),
                      ),
                    const Divider(),
                    if (flags.themeCustomization)
                      ListTile(
                        leading: const Icon(Icons.palette_outlined),
                        title: const Text('Appearance'),
                        subtitle: const Text(
                            'Theme, color, corner radius, density'),
                        trailing: const Icon(Icons.chevron_right),
                        onTap: () => context.push(AppRoutes.appearance),
                      ),
                    const Divider(),
                    ListTile(
                      leading: const Icon(Icons.logout, color: Colors.orange),
                      title: const Text('Sign Out'),
                      onTap: cubit.signOut,
                    ),
                    if (flags.deleteAccount) ...[
                      const Divider(),
                      ListTile(
                        leading: const Icon(Icons.delete_forever,
                            color: Colors.red),
                        title: const Text('Delete Account',
                            style: TextStyle(color: Colors.red)),
                        subtitle: const Text(
                            'Permanently remove your account and data'),
                        onTap: () => _showDeleteAccountDialog(context, cubit),
                      ),
                    ],
                  ],
                ),
        );
      },
    );
  }

  void _showTwoFactorQrDialog(
      BuildContext context, String uri, String qrCode) {
    final codeController = TextEditingController();
    final cubit = context.read<SettingsCubit>();
    showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Enable Two-Factor Auth'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              '1. Open your authenticator app (Google Authenticator, Authy, etc.).',
            ),
            const SizedBox(height: 8),
            const Text('2. Scan this URI or enter it manually:'),
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: Colors.grey.shade100,
                borderRadius: BorderRadius.circular(4),
              ),
              child: Row(
                children: [
                  Expanded(
                    child: SelectableText(uri,
                        style: const TextStyle(fontSize: 10)),
                  ),
                  IconButton(
                    icon: const Icon(Icons.copy, size: 16),
                    onPressed: () =>
                        Clipboard.setData(ClipboardData(text: uri)),
                    tooltip: 'Copy URI',
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
            const Text('3. Enter the 6-digit code to confirm setup:'),
            const SizedBox(height: 8),
            TextField(
              controller: codeController,
              keyboardType: TextInputType.number,
              maxLength: 6,
              textAlign: TextAlign.center,
              inputFormatters: [FilteringTextInputFormatter.digitsOnly],
              decoration: const InputDecoration(
                hintText: '------',
                border: OutlineInputBorder(),
                counterText: '',
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () {
              Navigator.of(dialogContext).pop();
              cubit.resetToInitial();
            },
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              final code = codeController.text.trim();
              if (code.length == 6) {
                Navigator.of(dialogContext).pop();
                cubit.verifyTwoFactorSetup(code: code);
              }
            },
            child: const Text('Confirm'),
          ),
        ],
      ),
    );
  }

  void _showDisableTwoFactorDialog(
      BuildContext context, SettingsCubit cubit) {
    final codeController = TextEditingController();
    showDialog<void>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Disable Two-Factor Auth'),
        content: TextField(
          controller: codeController,
          keyboardType: TextInputType.number,
          maxLength: 6,
          textAlign: TextAlign.center,
          inputFormatters: [FilteringTextInputFormatter.digitsOnly],
          decoration: const InputDecoration(
            labelText: '6-digit TOTP code',
            border: OutlineInputBorder(),
            counterText: '',
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            onPressed: () {
              final code = codeController.text.trim();
              if (code.isNotEmpty) {
                Navigator.of(dialogContext).pop();
                cubit.disableTwoFactor(code: code);
              }
            },
            child: const Text('Disable',
                style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
  }

  void _showDeleteAccountDialog(BuildContext context, SettingsCubit cubit) {
    final passwordController = TextEditingController();
    final confirmController = TextEditingController();
    showDialog<void>(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (dialogContext, setState) => AlertDialog(
          title: const Text('Delete Account',
              style: TextStyle(color: Colors.red)),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Text(
                'This action is permanent. Your account, profile data, and '
                'organizations you own will be deleted.',
              ),
              const SizedBox(height: 16),
              TextField(
                controller: passwordController,
                obscureText: true,
                onChanged: (_) => setState(() {}),
                decoration: const InputDecoration(
                  labelText: 'Current password',
                ),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: confirmController,
                onChanged: (_) => setState(() {}),
                decoration: const InputDecoration(
                  labelText: 'Type DELETE to confirm',
                ),
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(dialogContext).pop(),
              child: const Text('Cancel'),
            ),
            ElevatedButton(
              style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
              onPressed: confirmController.text.trim() == 'DELETE' &&
                      passwordController.text.isNotEmpty
                  ? () {
                      Navigator.of(dialogContext).pop();
                      cubit.deleteAccount(password: passwordController.text);
                    }
                  : null,
              child: const Text('Delete Forever',
                  style: TextStyle(color: Colors.white)),
            ),
          ],
        ),
      ),
    );
  }

  void _showChangePasswordDialog(
      BuildContext context, SettingsCubit cubit) {
    final currentPassController = TextEditingController();
    final newPassController = TextEditingController();
    showDialog<void>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Change Password'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: currentPassController,
              obscureText: true,
              decoration:
                  const InputDecoration(labelText: 'Current Password'),
            ),
            const SizedBox(height: 16),
            TextField(
              controller: newPassController,
              obscureText: true,
              decoration: const InputDecoration(labelText: 'New Password'),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              Navigator.of(dialogContext).pop();
              cubit.changePassword(
                currentPassword: currentPassController.text,
                newPassword: newPassController.text,
              );
            },
            child: const Text('Change'),
          ),
        ],
      ),
    );
  }
}
