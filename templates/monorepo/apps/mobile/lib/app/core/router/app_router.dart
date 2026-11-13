import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../../modules/auth/auth_bloc.dart';
import '../../modules/auth/continue_with/continue_with_cubit.dart';
import '../../modules/auth/oidc/oidc_cubit.dart';
import '../../modules/auth/oidc/oidc_view.dart';
import '../../modules/auth/sign_in/sign_in_cubit.dart';
import '../../modules/auth/sign_up/sign_up_cubit.dart';
import '../../modules/auth/forgot_password/forgot_password_cubit.dart';
import '../../modules/auth/reset_password/reset_password_cubit.dart';
import '../../modules/auth/verify_email/verify_email_cubit.dart';
import '../../modules/auth/two_factor/two_factor_cubit.dart';
import '../../modules/profile/profile_cubit.dart';
import '../../modules/settings/settings_cubit.dart';
import '../../modules/settings/linked_identities/linked_identities_cubit.dart';
import '../../modules/theme/theme_settings_view.dart';
import '../../modules/splash/splash_view.dart';
import '../../modules/auth/sign_in/sign_in_view.dart';
import '../../modules/auth/sign_up/sign_up_view.dart';
import '../../modules/auth/forgot_password/forgot_password_view.dart';
import '../../modules/auth/reset_password/reset_password_view.dart';
import '../../modules/auth/verify_email/verify_email_view.dart';
import '../../modules/auth/two_factor/two_factor_view.dart';
import '../../modules/home/home_view.dart';
import '../../modules/profile/profile_view.dart';
import '../../modules/settings/settings_view.dart';
import '../../data/repositories/auth.repository.dart';
import '../../data/repositories/user.repository.dart';
import '../../services/browser.dart';
import '../../routes/app_routes.dart';

// Organizations, magic-link sign-in and the admin panel have no routes:
// `apps/api` does not serve them yet (see docs/feature-flags.md).
GoRouter createRouter({
  required AuthBloc authBloc,
  required SharedPreferences prefs,
}) {
  return GoRouter(
    initialLocation: AppRoutes.splash,
    refreshListenable: _BlocChangeNotifier(authBloc.stream),
    redirect: (context, state) {
      final authState = authBloc.state;
      final location = state.uri.toString();

      final isLoading = authState is AuthInitial || authState is AuthLoading;
      final isAuthenticated = authState is AuthAuthenticated;

      // Still hydrating — stay on splash.
      if (isLoading) {
        return location == AppRoutes.splash ? null : AppRoutes.splash;
      }

      // A provider's deep link finishes a sign-in, or a signed-in account's
      // link: it opens whoever is signed in.
      if (state.uri.path == AppRoutes.oidc) return null;

      final isOnAuthRoute =
          _authRoutes.contains(location) ||
          location.startsWith(AppRoutes.resetPassword) ||
          location.startsWith(AppRoutes.verifyEmail);

      if (!isAuthenticated && !isOnAuthRoute) return AppRoutes.signIn;
      if (isAuthenticated && isOnAuthRoute) return AppRoutes.home;

      return null;
    },
    routes: [
      GoRoute(
        path: AppRoutes.splash,
        builder: (context, state) => const SplashView(),
      ),

      // ── Auth routes (guest-only) ──────────────────────────────────────────
      GoRoute(
        path: AppRoutes.signIn,
        builder: (context, _) => MultiBlocProvider(
          providers: [
            BlocProvider(
              create: (_) =>
                  SignInCubit(authRepository: context.read<AuthRepository>()),
            ),
            BlocProvider(
              create: (_) => ContinueWithCubit(
                authRepository: context.read<AuthRepository>(),
                browser: context.read<Browser>(),
              )..load(),
            ),
          ],
          child: const SignInView(),
        ),
      ),
      GoRoute(
        path: AppRoutes.signUp,
        builder: (context, _) => BlocProvider(
          create: (_) =>
              SignUpCubit(authRepository: context.read<AuthRepository>()),
          child: const SignUpView(),
        ),
      ),
      GoRoute(
        path: AppRoutes.forgotPassword,
        builder: (context, _) => BlocProvider(
          create: (_) => ForgotPasswordCubit(
            authRepository: context.read<AuthRepository>(),
          ),
          child: const ForgotPasswordView(),
        ),
      ),
      GoRoute(
        path: AppRoutes.resetPassword,
        builder: (context, state) {
          final token = state.uri.queryParameters['token'] ?? '';
          return BlocProvider(
            create: (_) => ResetPasswordCubit(
              authRepository: context.read<AuthRepository>(),
            ),
            child: ResetPasswordView(token: token),
          );
        },
      ),
      GoRoute(
        path: AppRoutes.verifyEmail,
        builder: (context, state) {
          final token = state.uri.queryParameters['token'] ?? '';
          return BlocProvider(
            create: (_) => VerifyEmailCubit(
              authRepository: context.read<AuthRepository>(),
            ),
            child: VerifyEmailView(token: token),
          );
        },
      ),
      // The deep link `<APP_SCHEME>://auth/oidc?…` (see `deep_links.dart`).
      GoRoute(
        path: AppRoutes.oidc,
        builder: (context, state) => BlocProvider(
          create: (_) =>
              OidcCubit(authRepository: context.read<AuthRepository>()),
          child: OidcView(parameters: state.uri.queryParameters),
        ),
      ),
      // Reached from sign-in only: the pending token travels as `extra`.
      GoRoute(
        path: AppRoutes.twoFactor,
        redirect: (context, state) =>
            state.extra is String ? null : AppRoutes.signIn,
        builder: (context, state) => BlocProvider(
          create: (_) => TwoFactorCubit(
            authRepository: context.read<AuthRepository>(),
            pendingToken: state.extra! as String,
          ),
          child: const TwoFactorView(),
        ),
      ),

      // ── Protected routes (auth-required) ─────────────────────────────────
      GoRoute(
        path: AppRoutes.home,
        builder: (context, state) => const HomeView(),
      ),
      GoRoute(
        path: AppRoutes.profile,
        builder: (context, _) => BlocProvider(
          create: (_) => ProfileCubit(
            userRepository: context.read<UserRepository>(),
            authRepository: context.read<AuthRepository>(),
            authBloc: context.read<AuthBloc>(),
          )..loadFromAuthBloc(),
          child: const ProfileView(),
        ),
      ),
      GoRoute(
        path: AppRoutes.settings,
        builder: (context, _) => MultiBlocProvider(
          providers: [
            BlocProvider(
              create: (_) => SettingsCubit(
                authRepository: context.read<AuthRepository>(),
                userRepository: context.read<UserRepository>(),
                authBloc: context.read<AuthBloc>(),
                prefs: prefs,
              ),
            ),
            BlocProvider(
              create: (_) => LinkedIdentitiesCubit(
                authRepository: context.read<AuthRepository>(),
                browser: context.read<Browser>(),
              )..load(),
            ),
          ],
          child: const SettingsView(),
        ),
      ),
      GoRoute(
        path: AppRoutes.appearance,
        builder: (context, _) => const ThemeSettingsView(),
      ),
    ],
  );
}

/// Auth routes that unauthenticated users are allowed to visit.
const _authRoutes = {
  AppRoutes.signIn,
  AppRoutes.signUp,
  AppRoutes.forgotPassword,
  AppRoutes.twoFactor,
};

/// Bridges a BLoC [Stream] to [ChangeNotifier] so GoRouter's
/// [refreshListenable] re-evaluates redirects on every auth state change.
class _BlocChangeNotifier extends ChangeNotifier {
  _BlocChangeNotifier(Stream<dynamic> stream) {
    _subscription = stream.listen((_) => notifyListeners());
  }

  late final StreamSubscription<dynamic> _subscription;

  @override
  void dispose() {
    _subscription.cancel();
    super.dispose();
  }
}
