import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/utils/secure_screen.dart';
import '../../../core/utils/snackbar_helper.dart';
import '../../../routes/app_routes.dart';
import 'verify_email_cubit.dart';

class VerifyEmailView extends StatefulWidget {
  const VerifyEmailView({super.key, required this.token});
  final String token;

  @override
  State<VerifyEmailView> createState() => _VerifyEmailViewState();
}

class _VerifyEmailViewState extends State<VerifyEmailView>
    with SecureScreenMixin {
  @override
  void initState() {
    super.initState();
    if (widget.token.isNotEmpty) {
      context.read<VerifyEmailCubit>().verify(token: widget.token);
    }
  }

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<VerifyEmailCubit, VerifyEmailState>(
      listener: (context, state) {
        if (state is VerifyEmailFailure) {
          SnackbarHelper.showError(context, state.message);
        } else if (state is VerifyEmailResent) {
          SnackbarHelper.showSuccess(context, 'Verification email sent!');
        }
      },
      builder: (context, state) {
        final isVerified = state is VerifyEmailSuccess;
        final isLoading = state is VerifyEmailLoading;

        return Scaffold(
          appBar: AppBar(title: const Text('Verify Email')),
          body: Center(
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(
                    isVerified ? Icons.check_circle : Icons.mark_email_unread,
                    size: 100,
                    color: isVerified ? Colors.green : Colors.grey,
                  ),
                  const SizedBox(height: 32),
                  Text(
                    isVerified ? 'Email Verified!' : 'Verify Your Email',
                    style: const TextStyle(
                        fontSize: 24, fontWeight: FontWeight.bold),
                  ),
                  const SizedBox(height: 16),
                  Text(
                    isVerified
                        ? 'Your email has been verified successfully.'
                        : 'Please check your email and click the verification link.',
                    style: const TextStyle(color: Colors.grey),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 48),
                  if (isLoading)
                    const CircularProgressIndicator()
                  else if (isVerified)
                    ElevatedButton(
                      onPressed: () => context.go(AppRoutes.signIn),
                      child: const Text('Continue'),
                    )
                  else ...[
                    ElevatedButton(
                      onPressed: context
                          .read<VerifyEmailCubit>()
                          .resendEmail,
                      child: const Text('Resend Email'),
                    ),
                    const SizedBox(height: 12),
                    TextButton(
                      onPressed: () => context.go(AppRoutes.signIn),
                      child: const Text('Go to Sign In'),
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
