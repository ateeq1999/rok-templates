import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/utils/snackbar_helper.dart';
import '../../../routes/app_routes.dart';
import '../auth_bloc.dart';
import 'magic_link_cubit.dart';

class MagicLinkView extends StatefulWidget {
  const MagicLinkView({super.key, this.token});

  /// Non-null when arriving via deep link (e.g. /magic-link?token=xxx).
  final String? token;

  @override
  State<MagicLinkView> createState() => _MagicLinkViewState();
}

class _MagicLinkViewState extends State<MagicLinkView> {
  final _emailController = TextEditingController();

  @override
  void initState() {
    super.initState();
    if (widget.token != null && widget.token!.isNotEmpty) {
      context.read<MagicLinkCubit>().verifyLink(token: widget.token!);
    }
  }

  @override
  void dispose() {
    _emailController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<MagicLinkCubit, MagicLinkState>(
      listener: (context, state) {
        if (state is MagicLinkFailure) {
          SnackbarHelper.showError(context, state.message);
        } else if (state is MagicLinkVerified) {
          if (state.token != null) {
            context
                .read<AuthBloc>()
                .add(AuthUserChanged(user: state.user, token: state.token!));
            context.go(AppRoutes.home);
          } else {
            SnackbarHelper.showError(
                context, 'Please verify your email before signing in.');
          }
        }
      },
      builder: (context, state) {
        final isSent = state is MagicLinkSent;
        final isLoading = state is MagicLinkLoading;
        final isVerifying =
            isLoading && widget.token != null && widget.token!.isNotEmpty;

        if (isVerifying) {
          return const Scaffold(
            body: Center(child: CircularProgressIndicator()),
          );
        }

        return Scaffold(
          appBar: AppBar(title: const Text('Sign in with Magic Link')),
          body: Center(
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(
                    isSent ? Icons.mark_email_read : Icons.auto_awesome,
                    size: 80,
                    color: isSent ? Colors.green : Colors.blue,
                  ),
                  const SizedBox(height: 32),
                  Text(
                    isSent ? 'Check Your Email' : 'Magic Link Sign In',
                    style: const TextStyle(
                        fontSize: 24, fontWeight: FontWeight.bold),
                  ),
                  const SizedBox(height: 16),
                  if (state is MagicLinkSent) ...[
                    Text(
                      'We sent a sign-in link to ${state.email}. '
                      'Tap the link in your email to sign in.',
                      style: const TextStyle(color: Colors.grey),
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 48),
                    TextButton(
                      onPressed: () => context.go(AppRoutes.signIn),
                      child: const Text('Back to Sign In'),
                    ),
                  ] else ...[
                    const Text(
                      'Enter your email and we\'ll send you a one-time sign-in link.',
                      style: TextStyle(color: Colors.grey),
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 32),
                    TextField(
                      controller: _emailController,
                      keyboardType: TextInputType.emailAddress,
                      decoration: const InputDecoration(
                        labelText: 'Email',
                        border: OutlineInputBorder(),
                      ),
                    ),
                    const SizedBox(height: 24),
                    if (isLoading)
                      const CircularProgressIndicator()
                    else
                      SizedBox(
                        width: double.infinity,
                        child: ElevatedButton(
                          onPressed: () {
                            final email = _emailController.text.trim();
                            if (email.isNotEmpty) {
                              context
                                  .read<MagicLinkCubit>()
                                  .sendLink(email: email);
                            }
                          },
                          child: const Text('Send Magic Link'),
                        ),
                      ),
                    const SizedBox(height: 12),
                    TextButton(
                      onPressed: () => context.go(AppRoutes.signIn),
                      child: const Text('Back to Sign In'),
                    ),
                  ],
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}
