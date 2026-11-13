import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';

import 'package:mobile/app/core/config/feature_flags.dart';
import 'package:mobile/app/core/router/app_router.dart';
import 'package:mobile/app/core/router/deep_links.dart';
import 'package:mobile/app/data/repositories/auth.repository.dart';
import 'package:mobile/app/data/repositories/user.repository.dart';
import 'package:mobile/app/modules/auth/auth_bloc.dart';
import 'package:mobile/app/services/auth_service.dart';
import 'package:mobile/app/services/browser.dart';

import '../support/api.dart';

/// A browser that only records where it was sent.
class RecordingBrowser implements Browser {
  final opened = <Uri>[];

  @override
  Future<void> open(Uri url) async => opened.add(url);
}

void main() {
  late AuthService session;
  late Requests requests;
  late RecordingBrowser browser;

  setUp(() {
    FlutterSecureStorage.setMockInitialValues({});
    SharedPreferences.setMockInitialValues({});
    session = AuthService();
    requests = [];
    browser = RecordingBrowser();
  });

  tearDown(() => session.dispose());

  /// The App's router and screens over an API that answers `answer`.
  Future<GoRouter> pumpApp(
    WidgetTester tester,
    Future<http.Response> Function(http.Request request) answer,
  ) async {
    final api = fakeApi(answer, session: session, requests: requests);
    final authBloc = AuthBloc(authService: session);
    final router = createRouter(
      authBloc: authBloc,
      prefs: await SharedPreferences.getInstance(),
    );
    await tester.pumpWidget(
      MultiRepositoryProvider(
        providers: [
          RepositoryProvider<AuthService>.value(value: session),
          RepositoryProvider<AuthRepository>.value(
            value: AuthRepository(api, session, apiUrl: apiOrigin),
          ),
          RepositoryProvider<UserRepository>.value(value: UserRepository(api)),
          RepositoryProvider<FeatureFlags>.value(
            value: const FeatureFlags(
              twoFactor: true,
              signUp: true,
              themeCustomization: true,
            ),
          ),
          RepositoryProvider<Browser>.value(value: browser),
        ],
        child: BlocProvider<AuthBloc>.value(
          value: authBloc,
          child: MaterialApp.router(routerConfig: router),
        ),
      ),
    );
    await tester.pumpAndSettle();
    return router;
  }

  testWidgets('signs in with a provider from the deep link it comes back by',
      (tester) async {
    final router = await pumpApp(
      tester,
      (request) async => switch (request.url.path) {
        '/api/v1/auth/oidc/providers' => json([
            {'name': 'google'},
          ]),
        '/api/v1/auth/oidc/handoff' => json({
            'access_token': 'tok-1',
            'expires_in': 3600,
            'token_type': 'Bearer',
          }),
        '/api/v1/auth/me' => json(userJson()),
        _ => json(null, status: 404),
      },
    );

    await tester.tap(find.text('Continue with Google'));
    await tester.pumpAndSettle();
    expect(browser.opened, [
      Uri.parse('$apiOrigin/api/v1/auth/oidc/google/start?client=mobile'),
    ]);

    // The provider is done: the API's callback opens the App's deep link.
    router.go(deepLinkLocation(Uri.parse('myapp://auth/oidc?code=hand-over'))!);
    await tester.pumpAndSettle();

    final handoff = requests.singleWhere(
      (request) => request.url.path == '/api/v1/auth/oidc/handoff',
    );
    expect(jsonDecode(handoff.body), {'code': 'hand-over'});
    expect(await session.getToken(), 'tok-1');
    expect(router.routerDelegate.currentConfiguration.uri.path, '/home');
  });

  testWidgets('shows no provider button when none is configured',
      (tester) async {
    await pumpApp(
      tester,
      (request) async => switch (request.url.path) {
        '/api/v1/auth/oidc/providers' => json(<Object?>[]),
        _ => json(null, status: 404),
      },
    );

    expect(find.textContaining('Continue with'), findsNothing);
    expect(find.text('Sign In'), findsWidgets);
  });

  testWidgets('a refused sign-in says why', (tester) async {
    final router = await pumpApp(
      tester,
      (request) async => json(<Object?>[]),
    );

    router.go(deepLinkLocation(Uri.parse(
      'myapp://auth/oidc?error=email_in_use&message=an+account+already+uses+it',
    ))!);
    await tester.pumpAndSettle();

    expect(find.text('an account already uses it'), findsOneWidget);
    expect(await session.getToken(), isNull);
  });
}
