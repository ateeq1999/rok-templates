import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/snackbar_helper.dart';
import '../../auth/oidc/provider_label.dart';
import 'linked_identities_cubit.dart';

/// The settings' "Linked accounts": each identity with Unlink, and Link for
/// each provider not linked yet. Hidden while no provider is configured.
class LinkedIdentitiesSection extends StatelessWidget {
  const LinkedIdentitiesSection({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<LinkedIdentitiesCubit, LinkedIdentitiesState>(
      listenWhen: (_, next) => next.message != null,
      listener: (context, state) =>
          SnackbarHelper.showError(context, state.message!),
      builder: (context, state) {
        if (state.providers.isEmpty) return const SizedBox.shrink();
        final cubit = context.read<LinkedIdentitiesCubit>();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const ListTile(title: Text('Linked accounts')),
            for (final identity in state.identities)
              ListTile(
                leading: const Icon(Icons.link),
                title: Text(providerLabel(identity.provider)),
                subtitle: identity.email == null ? null : Text(identity.email!),
                trailing: TextButton(
                  onPressed: () => cubit.unlink(identity),
                  child: const Text('Unlink'),
                ),
              ),
            for (final provider in state.linkable)
              ListTile(
                leading: const Icon(Icons.add_link),
                title: Text('Link ${providerLabel(provider)}'),
                onTap: () => cubit.link(provider),
              ),
            const Divider(),
          ],
        );
      },
    );
  }
}
