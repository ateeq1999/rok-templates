import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/utils/snackbar_helper.dart';
import '../../../routes/app_routes.dart';
import '../auth_bloc.dart';
import 'oidc_cubit.dart';
import 'provider_label.dart';

/// The screen the deep link `<APP_SCHEME>://auth/oidc?…` opens: it finishes
/// the sign-in (or the link) with a provider, then moves on.
class OidcView extends StatefulWidget {
  const OidcView({super.key, required this.parameters});

  /// The deep link's query parameters.
  final Map<String, String> parameters;

  @override
  State<OidcView> createState() => _OidcViewState();
}

class _OidcViewState extends State<OidcView> {
  @override
  void initState() {
    super.initState();
    context.read<OidcCubit>().finish(widget.parameters);
  }

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<OidcCubit, OidcState>(
      listener: (context, state) {
        switch (state) {
          case OidcSignedIn(:final user, :final token):
            context
                .read<AuthBloc>()
                .add(AuthUserChanged(user: user, token: token));
            context.go(AppRoutes.home);
          case OidcTwoFactorRequired(:final pendingToken):
            context.go(AppRoutes.twoFactor, extra: pendingToken);
          case OidcLinked(:final provider):
            SnackbarHelper.showSuccess(
              context,
              '${providerLabel(provider)} is linked',
            );
            context.go(AppRoutes.settings);
          case OidcFinishing() || OidcRefused():
            break;
        }
      },
      builder: (context, state) => Scaffold(
        appBar: AppBar(title: const Text('Sign In')),
        body: Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: switch (state) {
              OidcRefused(:final message) => Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(message, textAlign: TextAlign.center),
                    const SizedBox(height: 24),
                    TextButton(
                      onPressed: () => context.go(AppRoutes.signIn),
                      child: const Text('Back to Sign In'),
                    ),
                  ],
                ),
              _ => const CircularProgressIndicator(),
            },
          ),
        ),
      ),
    );
  }
}
