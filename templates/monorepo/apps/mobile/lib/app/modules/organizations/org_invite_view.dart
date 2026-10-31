import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../core/utils/snackbar_helper.dart';
import 'org_invitation_cubit.dart';

class OrgInviteView extends StatefulWidget {
  const OrgInviteView({super.key, required this.orgId});

  final String orgId;

  @override
  State<OrgInviteView> createState() => _OrgInviteViewState();
}

class _OrgInviteViewState extends State<OrgInviteView> {
  final _emailController = TextEditingController();
  String _role = 'member';

  @override
  void dispose() {
    _emailController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<OrgInvitationCubit, OrgInvitationState>(
      listener: (context, state) {
        if (state is OrgInvitationSent) {
          SnackbarHelper.showSuccess(
              context, 'Invitation sent to ${state.invitation.email}');
          Navigator.of(context).pop();
        } else if (state is OrgInvitationFailure) {
          SnackbarHelper.showError(context, state.message);
        }
      },
      builder: (context, state) {
        final isLoading = state is OrgInvitationLoading;

        return Scaffold(
          appBar: AppBar(title: const Text('Invite Member')),
          body: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                TextField(
                  controller: _emailController,
                  keyboardType: TextInputType.emailAddress,
                  decoration: const InputDecoration(
                    labelText: 'Email Address',
                    border: OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 16),
                DropdownButtonFormField<String>(
                  initialValue: _role,
                  decoration: const InputDecoration(
                    labelText: 'Role',
                    border: OutlineInputBorder(),
                  ),
                  items: const [
                    DropdownMenuItem(value: 'member', child: Text('Member')),
                    DropdownMenuItem(value: 'admin', child: Text('Admin')),
                  ],
                  onChanged: (v) => setState(() => _role = v ?? 'member'),
                ),
                const SizedBox(height: 24),
                if (isLoading)
                  const Center(child: CircularProgressIndicator())
                else
                  ElevatedButton(
                    onPressed: () {
                      final email = _emailController.text.trim();
                      if (email.isNotEmpty) {
                        context
                            .read<OrgInvitationCubit>()
                            .inviteMember(widget.orgId, email, _role);
                      }
                    },
                    child: const Text('Send Invitation'),
                  ),
              ],
            ),
          ),
        );
      },
    );
  }
}
