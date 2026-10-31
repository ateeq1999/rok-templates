import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/utils/secure_screen.dart';
import '../../../core/utils/snackbar_helper.dart';
import '../../../routes/app_routes.dart';
import 'reset_password_cubit.dart';

class ResetPasswordView extends StatefulWidget {
  const ResetPasswordView({super.key, required this.token});
  final String token;

  @override
  State<ResetPasswordView> createState() => _ResetPasswordViewState();
}

class _ResetPasswordViewState extends State<ResetPasswordView>
    with SecureScreenMixin {
  final _formKey = GlobalKey<FormState>();
  final _passwordController = TextEditingController();
  final _confirmPasswordController = TextEditingController();

  @override
  void dispose() {
    _passwordController.dispose();
    _confirmPasswordController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<ResetPasswordCubit, ResetPasswordState>(
      listener: (context, state) {
        if (state is ResetPasswordSuccess) {
          SnackbarHelper.showSuccess(context, 'Password reset successful!');
          context.go(AppRoutes.signIn);
        } else if (state is ResetPasswordFailure) {
          final hasFieldErrors = state.fieldErrors?.isNotEmpty ?? false;
          if (!hasFieldErrors) {
            SnackbarHelper.showError(context, state.message);
          }
        }
      },
      builder: (context, state) {
        final cubit = context.read<ResetPasswordCubit>();
        final isLoading = state is ResetPasswordLoading;
        final fieldErrors =
            state is ResetPasswordFailure ? state.fieldErrors : null;

        return Scaffold(
          appBar: AppBar(title: const Text('Reset Password')),
          body: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: Form(
              key: _formKey,
              child: AutofillGroup(
                child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  const SizedBox(height: 40),
                  const Icon(Icons.lock_open, size: 80, color: Colors.grey),
                  const SizedBox(height: 24),
                  const Text(
                    'Set New Password',
                    style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 8),
                  const Text(
                    'Enter your new password below.',
                    style: TextStyle(color: Colors.grey),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 32),
                  BlocBuilder<ResetPasswordCubit, ResetPasswordState>(
                    builder: (context, _) => TextFormField(
                      controller: _passwordController,
                      obscureText: cubit.isPasswordHidden,
                      textInputAction: TextInputAction.next,
                      autofillHints: const [AutofillHints.newPassword],
                      forceErrorText: fieldErrors?['password'],
                      decoration: InputDecoration(
                        labelText: 'New Password',
                        border: const OutlineInputBorder(),
                        prefixIcon: const Icon(Icons.lock_outlined),
                        suffixIcon: IconButton(
                          icon: Icon(cubit.isPasswordHidden
                              ? Icons.visibility_off
                              : Icons.visibility),
                          onPressed: cubit.togglePasswordVisibility,
                        ),
                      ),
                      validator: (v) {
                        if (v == null || v.isEmpty) return 'Please enter a password';
                        if (v.length < 8) return 'Password must be at least 8 characters';
                        return null;
                      },
                    ),
                  ),
                  const SizedBox(height: 16),
                  TextFormField(
                    controller: _confirmPasswordController,
                    obscureText: true,
                    textInputAction: TextInputAction.done,
                    autofillHints: const [AutofillHints.newPassword],
                    decoration: const InputDecoration(
                      labelText: 'Confirm Password',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.lock_outlined),
                    ),
                    validator: (v) {
                      if (v == null || v.isEmpty) return 'Please confirm your password';
                      if (v != _passwordController.text) return 'Passwords do not match';
                      return null;
                    },
                  ),
                  const SizedBox(height: 24),
                  ElevatedButton(
                    onPressed: isLoading
                        ? null
                        : () {
                            if (_formKey.currentState!.validate()) {
                              cubit.reset(
                                token: widget.token,
                                password: _passwordController.text,
                              );
                            }
                          },
                    style: ElevatedButton.styleFrom(
                        padding: const EdgeInsets.symmetric(vertical: 16)),
                    child: isLoading
                        ? const SizedBox(
                            height: 20,
                            width: 20,
                            child: CircularProgressIndicator(
                                strokeWidth: 2, color: Colors.white),
                          )
                        : const Text('Reset Password'),
                  ),
                ],
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
