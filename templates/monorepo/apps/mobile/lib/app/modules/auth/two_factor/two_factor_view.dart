import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/utils/secure_screen.dart';
import '../../../core/utils/snackbar_helper.dart';
import '../../../routes/app_routes.dart';
import '../auth_bloc.dart';
import 'two_factor_cubit.dart';

class TwoFactorView extends StatefulWidget {
  const TwoFactorView({super.key});

  @override
  State<TwoFactorView> createState() => _TwoFactorViewState();
}

class _TwoFactorViewState extends State<TwoFactorView>
    with SecureScreenMixin {
  final _codeController = TextEditingController();

  @override
  void dispose() {
    _codeController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<TwoFactorCubit, TwoFactorState>(
      listener: (context, state) {
        if (state is TwoFactorSuccess && state.token != null) {
          Clipboard.setData(const ClipboardData(text: ''));
          context.read<AuthBloc>().add(
                AuthUserChanged(user: state.user, token: state.token!),
              );
          context.go(AppRoutes.home);
        } else if (state is TwoFactorFailure) {
          _codeController.clear();
          SnackbarHelper.showError(context, state.message);
        }
      },
      builder: (context, state) {
        final isLoading = state is TwoFactorLoading;

        return Scaffold(
          appBar: AppBar(title: const Text('Two-Factor Authentication')),
          body: Center(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Icon(Icons.security, size: 80, color: Colors.grey),
                  const SizedBox(height: 24),
                  const Text(
                    'Enter Verification Code',
                    style:
                        TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
                  ),
                  const SizedBox(height: 8),
                  const Text(
                    'Please enter the 6-digit code from your authenticator app.',
                    style: TextStyle(color: Colors.grey),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 32),
                  SizedBox(
                    width: 250,
                    child: TextField(
                      controller: _codeController,
                      keyboardType: TextInputType.number,
                      maxLength: 6,
                      textAlign: TextAlign.center,
                      style: const TextStyle(fontSize: 24, letterSpacing: 8),
                      inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                      decoration: const InputDecoration(
                        counterText: '',
                        hintText: '------',
                        border: OutlineInputBorder(),
                      ),
                      onChanged: (value) {
                        if (value.length == 6) {
                          context
                              .read<TwoFactorCubit>()
                              .verify(code: value);
                        }
                      },
                    ),
                  ),
                  const SizedBox(height: 32),
                  ElevatedButton(
                    onPressed: isLoading
                        ? null
                        : () => context
                            .read<TwoFactorCubit>()
                            .verify(code: _codeController.text),
                    style: ElevatedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(
                          vertical: 16, horizontal: 48),
                    ),
                    child: isLoading
                        ? const SizedBox(
                            height: 20,
                            width: 20,
                            child: CircularProgressIndicator(
                                strokeWidth: 2, color: Colors.white),
                          )
                        : const Text('Verify'),
                  ),
                  const SizedBox(height: 16),
                  TextButton(
                    onPressed: () => context.pop(),
                    child: const Text('Back to Sign In'),
                  ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}
