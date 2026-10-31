import 'dart:async';

import 'package:app_links/app_links.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_dotenv/flutter_dotenv.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:go_router/go_router.dart';
import 'package:hydrated_bloc/hydrated_bloc.dart';

import 'firebase_options.dart';
import 'app/core/config/feature_flags.dart';
import 'app/core/router/app_router.dart';
import 'app/core/theme/app_theme.dart';
import 'app/core/widgets/offline_banner.dart';
import 'app/modules/theme/theme_cubit.dart';
import 'app/data/providers/auth.provider.dart';
import 'app/data/providers/user.provider.dart';
import 'app/data/providers/admin.provider.dart';
import 'app/data/providers/organization.provider.dart';
import 'app/data/repositories/auth.repository.dart';
import 'app/data/repositories/user.repository.dart';
import 'app/data/repositories/admin.repository.dart';
import 'app/data/repositories/organization.repository.dart';
import 'app/modules/auth/auth_bloc.dart';
import 'app/routes/app_routes.dart';
import 'app/services/auth_service.dart';
import 'app/services/dio_service.dart';
import 'app/services/notification_service.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await Firebase.initializeApp(options: DefaultFirebaseOptions.currentPlatform);
  await dotenv.load(fileName: '.env');

  // HydratedBloc storage (persists bloc state across restarts).
  final storageDir = await getApplicationDocumentsDirectory();
  HydratedBloc.storage = await HydratedStorage.build(
    storageDirectory: HydratedStorageDirectory(storageDir.path),
  );

  // Register FCM background handler before runApp.
  FirebaseMessaging.onBackgroundMessage(firebaseMessagingBackgroundHandler);

  final prefs = await SharedPreferences.getInstance();
  final featureFlags = FeatureFlags.fromEnv();

  // ── Dependency graph (bottom-up) ─────────────────────────────────────────
  final authService = AuthService();
  final dioService = DioService(authService);
  final authProvider = AuthProvider(dioService.dio);
  final userProvider = UserProvider(dioService.dio);
  final adminProvider = AdminProvider(dioService.dio);
  final orgProvider = OrganizationProvider(dioService.dio);
  final authRepository = AuthRepository(authProvider);
  final userRepository = UserRepository(userProvider);
  final adminRepository = AdminRepository(adminProvider);
  final orgRepository = OrganizationRepository(orgProvider);
  final notificationService = NotificationService(userRepository: userRepository);

  await notificationService.init();

  runApp(
    MyApp(
      authService: authService,
      authRepository: authRepository,
      userRepository: userRepository,
      adminRepository: adminRepository,
      orgRepository: orgRepository,
      notificationService: notificationService,
      featureFlags: featureFlags,
      prefs: prefs,
    ),
  );
}

class MyApp extends StatefulWidget {
  const MyApp({
    super.key,
    required this.authService,
    required this.authRepository,
    required this.userRepository,
    required this.adminRepository,
    required this.orgRepository,
    required this.notificationService,
    required this.featureFlags,
    required this.prefs,
  });

  final AuthService authService;
  final AuthRepository authRepository;
  final UserRepository userRepository;
  final AdminRepository adminRepository;
  final OrganizationRepository orgRepository;
  final NotificationService notificationService;
  final FeatureFlags featureFlags;
  final SharedPreferences prefs;

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  late final AuthBloc _authBloc;
  late final _GoRouterHolder _routerHolder;
  StreamSubscription<Uri>? _deepLinkSub;

  @override
  void initState() {
    super.initState();
    _authBloc = AuthBloc(authService: widget.authService);

    _routerHolder = _GoRouterHolder(
      authBloc: _authBloc,
      authRepository: widget.authRepository,
      userRepository: widget.userRepository,
      authService: widget.authService,
      prefs: widget.prefs,
    );

    // Wire notification router after GoRouter is created.
    widget.notificationService.router = _routerHolder.router;

    _initDeepLinks();
  }

  void _initDeepLinks() {
    final appLinks = AppLinks();
    _deepLinkSub = appLinks.uriLinkStream.listen(
      _handleDeepLink,
      onError: (err) => debugPrint('Deep link error: $err'),
    );
  }

  void _handleDeepLink(Uri uri) {
    final scheme = dotenv.env['APP_SCHEME'] ?? 'myapp';
    if (uri.scheme != scheme && uri.scheme != 'myapp') return;

    final token = uri.queryParameters['token'];
    final router = _routerHolder.router;

    if (uri.host == 'invite' || uri.host == 'invitations') {
      if (token != null) {
        router.go('${AppRoutes.orgInvitationAccept}?token=$token');
      }
      return;
    }

    if (uri.host != 'auth') return;

    switch (uri.path) {
      case '/callback':
        if (token != null) {
          // OAuth callback — save token and navigate home.
          widget.authService.saveToken(token).then((_) {
            router.go(AppRoutes.home);
          });
        }
      case '/verify-email':
        if (token != null) {
          router.go('${AppRoutes.verifyEmail}?token=$token');
        }
      case '/reset-password':
        if (token != null) {
          router.go('${AppRoutes.resetPassword}?token=$token');
        }
      case '/magic-link':
        if (token != null) {
          router.go('${AppRoutes.magicLink}?token=$token');
        }
    }
  }

  @override
  void dispose() {
    _deepLinkSub?.cancel();
    _authBloc.close();
    widget.authService.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<AuthService>.value(value: widget.authService),
        RepositoryProvider<AuthRepository>.value(value: widget.authRepository),
        RepositoryProvider<UserRepository>.value(value: widget.userRepository),
        RepositoryProvider<AdminRepository>.value(value: widget.adminRepository),
        RepositoryProvider<OrganizationRepository>.value(value: widget.orgRepository),
        RepositoryProvider<FeatureFlags>.value(value: widget.featureFlags),
      ],
      child: MultiBlocProvider(
        providers: [
          BlocProvider<AuthBloc>.value(value: _authBloc),
          BlocProvider<ThemeCubit>(create: (_) => ThemeCubit(widget.prefs)),
        ],
        child: BlocBuilder<ThemeCubit, ThemeState>(
          builder: (context, themeState) {
            final prefs = themeState.preferences;
            return MaterialApp.router(
              title: 'Flutter Starter',
              debugShowCheckedModeBanner: false,
              theme: AppTheme.build(prefs, Brightness.light),
              darkTheme: AppTheme.build(prefs, Brightness.dark),
              themeMode: prefs.themeMode,
              routerConfig: _routerHolder.router,
              builder: (context, child) =>
                  OfflineBanner(child: child ?? const SizedBox.shrink()),
            );
          },
        ),
      ),
    );
  }
}

/// Holds the [GoRouter] instance so it can be referenced before [build] runs.
class _GoRouterHolder {
  _GoRouterHolder({
    required AuthBloc authBloc,
    required AuthRepository authRepository,
    required UserRepository userRepository,
    required AuthService authService,
    required SharedPreferences prefs,
  }) : router = createRouter(
          authBloc: authBloc,
          authRepository: authRepository,
          userRepository: userRepository,
          authService: authService,
          prefs: prefs,
        );

  final GoRouter router;
}
