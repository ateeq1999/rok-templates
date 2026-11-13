import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../oidc/provider_label.dart';
import 'continue_with_cubit.dart';

/// "Continue with …" for each configured provider; nothing while
/// none is.
class ContinueWithButtons extends StatelessWidget {
  const ContinueWithButtons({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<ContinueWithCubit, List<String>>(
      builder: (context, providers) {
        if (providers.isEmpty) return const SizedBox.shrink();
        final cubit = context.read<ContinueWithCubit>();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            for (final provider in providers)
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: OutlinedButton(
                  onPressed: () => cubit.continueWith(provider),
                  style: OutlinedButton.styleFrom(
                    padding: const EdgeInsets.symmetric(vertical: 16),
                  ),
                  child: Text('Continue with ${providerLabel(provider)}'),
                ),
              ),
          ],
        );
      },
    );
  }
}
