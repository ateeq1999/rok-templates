import 'dart:async';

import 'package:api_client/api_client.dart';
import 'package:app_links/app_links.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:go_router/go_router.dart';
import 'package:hydrated_bloc/hydrated_bloc.dart';

import 'app/core/config/app_config.dart';
import 'app/core/config/feature_flags.dart';
import 'app/core/router/app_router.dart';
import 'app/core/theme/app_theme.dart';
import 'app/core/widgets/offline_banner.dart';
import 'app/modules/theme/theme_cubit.dart';
import 'app/data/repositories/auth.repository.dart';
import 'app/data/repositories/user.repository.dart';
import 'app/modules/auth/auth_bloc.dart';
import 'app/routes/app_routes.dart';
import 'app/services/auth_service.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // HydratedBloc storage (persists bloc state across restarts).
  final storageDir = await getApplicationDocumentsDirectory();
  HydratedBloc.storage = await HydratedStorage.build(
    storageDirectory: HydratedStorageDirectory(storageDir.path),
  );

  final prefs = await SharedPreferences.getInstance();
  final featureFlags = FeatureFlags.fromEnvironment();

  // ── Dependency graph (bottom-up) ─────────────────────────────────────────
  // The generated client (`clients/dart`) is the App's only way to the API;
  // the session store gives it the bearer token and hears when it expires.
  final authService = AuthService();
  final api = ApiClient(baseUrl: AppConfig.apiUrl, session: authService);
  final authRepository = AuthRepository(api, authService);
  final userRepository = UserRepository(api);

  runApp(
    MyApp(
      authService: authService,
      authRepository: authRepository,
      userRepository: userRepository,
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
    required this.featureFlags,
    required this.prefs,
  });

  final AuthService authService;
  final AuthRepository authRepository;
  final UserRepository userRepository;
  final FeatureFlags featureFlags;
  final SharedPreferences prefs;

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  late final AuthBloc _authBloc;
  late final GoRouter _router;
  StreamSubscription<Uri>? _deepLinkSub;

  @override
  void initState() {
    super.initState();
    _authBloc = AuthBloc(authService: widget.authService);
    _router = createRouter(authBloc: _authBloc, prefs: widget.prefs);
    _initDeepLinks();
  }

  void _initDeepLinks() {
    final appLinks = AppLinks();
    _deepLinkSub = appLinks.uriLinkStream.listen(
      _handleDeepLink,
      onError: (err) => debugPrint('Deep link error: $err'),
    );
  }

  /// `<scheme>://auth/verify-email?token=...` and
  /// `<scheme>://auth/reset-password?token=...` open the screens that use
  /// the token the API mailed.
  void _handleDeepLink(Uri uri) {
    if (uri.scheme != AppConfig.appScheme || uri.host != 'auth') return;
    final token = uri.queryParameters['token'];
    if (token == null) return;

    switch (uri.path) {
      case '/verify-email':
        _router.go(Uri(
          path: AppRoutes.verifyEmail,
          queryParameters: {'token': token},
        ).toString());
      case '/reset-password':
        _router.go(Uri(
          path: AppRoutes.resetPassword,
          queryParameters: {'token': token},
        ).toString());
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
              routerConfig: _router,
              builder: (context, child) =>
                  OfflineBanner(child: child ?? const SizedBox.shrink()),
            );
          },
        ),
      ),
    );
  }
}
