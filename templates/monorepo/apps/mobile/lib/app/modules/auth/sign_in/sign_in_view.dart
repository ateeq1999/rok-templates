import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/config/feature_flags.dart';
import '../../../core/utils/snackbar_helper.dart';
import '../../../routes/app_routes.dart';
import '../auth_bloc.dart';
import 'sign_in_cubit.dart';

class SignInView extends StatefulWidget {
  const SignInView({super.key});

  @override
  State<SignInView> createState() => _SignInViewState();
}

class _SignInViewState extends State<SignInView> {
  final _formKey = GlobalKey<FormState>();
  final _emailController = TextEditingController();
  final _passwordController = TextEditingController();

  @override
  void dispose() {
    _emailController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  static bool _isEmail(String v) => RegExp(
        r'^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$',
      ).hasMatch(v);

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<SignInCubit, SignInState>(
      listener: (context, state) {
        if (state is SignInTwoFactorRequired) {
          context.push(AppRoutes.twoFactor);
        } else if (state is SignInSuccess) {
          if (state.token != null) {
            context.read<AuthBloc>().add(
                  AuthUserChanged(user: state.user, token: state.token!),
                );
            context.go(AppRoutes.home);
          } else {
            // Email verification pending — stay on sign-in, show message.
            SnackbarHelper.showError(
              context,
              'Please verify your email before signing in.',
            );
          }
        } else if (state is SignInFailure) {
          final hasFieldErrors = state.fieldErrors?.isNotEmpty ?? false;
          if (!hasFieldErrors) {
            SnackbarHelper.showError(context, state.message);
          }
        }
      },
      builder: (context, state) {
        final cubit = context.read<SignInCubit>();
        final isLoading = state is SignInLoading;
        final flags = context.read<FeatureFlags>();
        final fieldErrors =
            state is SignInFailure ? state.fieldErrors : null;

        return Scaffold(
          appBar: AppBar(title: const Text('Sign In')),
          body: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: Form(
              key: _formKey,
              child: AutofillGroup(
                child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  const SizedBox(height: 40),
                  TextFormField(
                    controller: _emailController,
                    keyboardType: TextInputType.emailAddress,
                    textInputAction: TextInputAction.next,
                    autofillHints: const [AutofillHints.username, AutofillHints.email],
                    forceErrorText: fieldErrors?['email'],
                    decoration: const InputDecoration(
                      labelText: 'Email',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.email_outlined),
                    ),
                    validator: (v) {
                      if (v == null || v.isEmpty) return 'Please enter your email';
                      if (!_isEmail(v)) return 'Please enter a valid email';
                      return null;
                    },
                  ),
                  const SizedBox(height: 16),
                  BlocBuilder<SignInCubit, SignInState>(
                    buildWhen: (prev, next) => true,
                    builder: (context, _) => TextFormField(
                      controller: _passwordController,
                      obscureText: cubit.isPasswordHidden,
                      textInputAction: TextInputAction.done,
                      autofillHints: const [AutofillHints.password],
                      forceErrorText: fieldErrors?['password'],
                      onFieldSubmitted: (_) {
                        if (_formKey.currentState!.validate()) {
                          cubit.signIn(
                            email: _emailController.text,
                            password: _passwordController.text,
                          );
                        }
                      },
                      decoration: InputDecoration(
                        labelText: 'Password',
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
                        if (v == null || v.isEmpty) return 'Please enter your password';
                        if (v.length < 6) return 'Password must be at least 6 characters';
                        return null;
                      },
                    ),
                  ),
                  const SizedBox(height: 8),
                  Align(
                    alignment: Alignment.centerRight,
                    child: TextButton(
                      onPressed: () => context.push(AppRoutes.forgotPassword),
                      child: const Text('Forgot Password?'),
                    ),
                  ),
                  const SizedBox(height: 24),
                  ElevatedButton(
                    onPressed: isLoading
                        ? null
                        : () {
                            if (_formKey.currentState!.validate()) {
                              cubit.signIn(
                                email: _emailController.text,
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
                        : const Text('Sign In'),
                  ),
                  const SizedBox(height: 24),
                  if (flags.signUp)
                    Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        const Text("Don't have an account?"),
                        TextButton(
                          onPressed: () => context.push(AppRoutes.signUp),
                          child: const Text('Sign Up'),
                        ),
                      ],
                    ),
                  if (flags.magicLink)
                    TextButton(
                      onPressed: () => context.push(AppRoutes.magicLink),
                      child: const Text('Sign in with Magic Link'),
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
