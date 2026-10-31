import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../modules/auth/auth_bloc.dart';
import '../../modules/auth/sign_in/sign_in_cubit.dart';
import '../../modules/auth/sign_up/sign_up_cubit.dart';
import '../../modules/auth/forgot_password/forgot_password_cubit.dart';
import '../../modules/auth/reset_password/reset_password_cubit.dart';
import '../../modules/auth/verify_email/verify_email_cubit.dart';
import '../../modules/auth/two_factor/two_factor_cubit.dart';
import '../../modules/auth/magic_link/magic_link_cubit.dart';
import '../../modules/auth/magic_link/magic_link_view.dart';
import '../../modules/profile/profile_cubit.dart';
import '../../modules/settings/settings_cubit.dart';
import '../../modules/theme/theme_settings_view.dart';
import '../../modules/splash/splash_view.dart';
import '../../modules/admin/admin_stats_cubit.dart';
import '../../modules/admin/admin_users_cubit.dart';
import '../../modules/admin/audit_logs_cubit.dart';
import '../../modules/admin/admin_dashboard_view.dart';
import '../../modules/admin/admin_users_view.dart';
import '../../modules/admin/admin_user_detail_view.dart';
import '../../modules/admin/audit_logs_view.dart';
import '../../modules/organizations/org_list_cubit.dart';
import '../../modules/organizations/org_detail_cubit.dart';
import '../../modules/organizations/org_invitation_cubit.dart';
import '../../modules/organizations/orgs_list_view.dart';
import '../../modules/organizations/org_detail_view.dart';
import '../../modules/organizations/org_invite_view.dart';
import '../../modules/organizations/invitation_accept_view.dart';
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
import '../../data/repositories/admin.repository.dart';
import '../../data/repositories/organization.repository.dart';
import '../../data/models/admin_user.model.dart';
import '../../services/auth_service.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../../routes/app_routes.dart';

GoRouter createRouter({
  required AuthBloc authBloc,
  required AuthRepository authRepository,
  required UserRepository userRepository,
  required AuthService authService,
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

      final isOnAuthRoute =
          _authRoutes.contains(location) ||
          location.startsWith(AppRoutes.resetPassword) ||
          location.startsWith(AppRoutes.verifyEmail);

      if (!isAuthenticated && !isOnAuthRoute) return AppRoutes.signIn;
      if (isAuthenticated && isOnAuthRoute) return AppRoutes.home;

      // Admin routes require role == 'admin'.
      if (isAuthenticated &&
          location.startsWith(AppRoutes.admin) &&
          authState.user.role != 'admin') {
        return AppRoutes.home;
      }

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
        builder: (context, _) => BlocProvider(
          create: (_) =>
              SignInCubit(authRepository: context.read<AuthRepository>()),
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
      GoRoute(
        path: AppRoutes.twoFactor,
        builder: (context, _) => BlocProvider(
          create: (_) =>
              TwoFactorCubit(authRepository: context.read<AuthRepository>()),
          child: const TwoFactorView(),
        ),
      ),
      GoRoute(
        path: AppRoutes.magicLink,
        builder: (context, state) {
          final token = state.uri.queryParameters['token'];
          return BlocProvider(
            create: (_) =>
                MagicLinkCubit(authRepository: context.read<AuthRepository>()),
            child: MagicLinkView(token: token),
          );
        },
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
            authBloc: context.read<AuthBloc>(),
          )..loadFromAuthBloc(),
          child: const ProfileView(),
        ),
      ),
      GoRoute(
        path: AppRoutes.settings,
        builder: (context, _) => BlocProvider(
          create: (_) => SettingsCubit(
            authRepository: context.read<AuthRepository>(),
            userRepository: context.read<UserRepository>(),
            authService: context.read<AuthService>(),
            authBloc: context.read<AuthBloc>(),
            prefs: prefs,
          ),
          child: const SettingsView(),
        ),
      ),
      GoRoute(
        path: AppRoutes.appearance,
        builder: (context, _) => const ThemeSettingsView(),
      ),

      // ── Admin routes (role=admin required — enforced in redirect) ────────
      GoRoute(
        path: AppRoutes.admin,
        builder: (context, _) => BlocProvider(
          create: (_) =>
              AdminStatsCubit(adminRepository: context.read<AdminRepository>()),
          child: const AdminDashboardView(),
        ),
      ),
      GoRoute(
        path: AppRoutes.adminUsers,
        builder: (context, _) => BlocProvider(
          create: (_) =>
              AdminUsersCubit(adminRepository: context.read<AdminRepository>()),
          child: const AdminUsersView(),
        ),
      ),
      GoRoute(
        path: '${AppRoutes.adminUsers}/:id',
        builder: (context, state) {
          final user = state.extra as AdminUser?;
          if (user == null) {
            return const Scaffold(
              body: Center(child: Text('User not available')),
            );
          }
          return BlocProvider(
            create: (_) => AdminUsersCubit(
              adminRepository: context.read<AdminRepository>(),
            ),
            child: AdminUserDetailView(user: user),
          );
        },
      ),
      GoRoute(
        path: AppRoutes.adminAuditLogs,
        builder: (context, _) => BlocProvider(
          create: (_) =>
              AuditLogsCubit(adminRepository: context.read<AdminRepository>()),
          child: const AuditLogsView(),
        ),
      ),

      // ── Organizations routes ─────────────────────────────────────────────
      GoRoute(
        path: AppRoutes.organizations,
        builder: (context, _) => BlocProvider(
          create: (_) => OrgListCubit(
            orgRepository: context.read<OrganizationRepository>(),
          ),
          child: const OrgsListView(),
        ),
      ),
      GoRoute(
        path: '${AppRoutes.organizations}/:id',
        builder: (context, state) {
          final orgId = state.pathParameters['id'];
          return BlocProvider(
            create: (_) => OrgDetailCubit(
              orgRepository: context.read<OrganizationRepository>(),
            ),
            child: OrgDetailView(orgId: orgId),
          );
        },
      ),
      GoRoute(
        path: '${AppRoutes.organizations}/:id/invite',
        builder: (context, state) {
          final orgId = state.pathParameters['id'];
          return BlocProvider(
            create: (_) => OrgInvitationCubit(
              orgRepository: context.read<OrganizationRepository>(),
            ),
            child: OrgInviteView(orgId: orgId),
          );
        },
      ),
      GoRoute(
        path: AppRoutes.orgInvitationAccept,
        builder: (context, state) {
          final token = state.uri.queryParameters['token'] ?? '';
          return InvitationAcceptView(
            token: token,
            orgRepository: context.read<OrganizationRepository>(),
          );
        },
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
  AppRoutes.magicLink,
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
