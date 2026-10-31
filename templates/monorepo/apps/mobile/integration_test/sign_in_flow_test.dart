import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:go_router/go_router.dart';

import 'package:mobile/app/data/repositories/auth.repository.dart';
import 'package:mobile/app/modules/auth/auth_bloc.dart';
import 'package:mobile/app/modules/auth/sign_in/sign_in_cubit.dart';
import 'package:mobile/app/modules/auth/sign_in/sign_in_view.dart';
import 'package:mobile/app/modules/home/home_view.dart';
import 'package:mobile/app/modules/auth/forgot_password/forgot_password_view.dart';
import 'package:mobile/app/modules/auth/sign_up/sign_up_view.dart';
import 'package:mobile/app/modules/auth/sign_up/sign_up_cubit.dart';
import 'package:mobile/app/services/auth_service.dart';
import 'package:mobile/app/routes/app_routes.dart';

import '../test/support/api.dart';

// ── Mocks ────────────────────────────────────────────────────────────────────

class MockAuthBloc extends MockBloc<AuthEvent, AuthState> implements AuthBloc {}

class MockAuthRepository extends Mock implements AuthRepository {}

class MockAuthService extends Mock implements AuthService {
  @override
  Stream<AuthStatus> get status => const Stream.empty();
}

// ── Helpers ──────────────────────────────────────────────────────────────────

final _testUser = testUser();

/// Builds a minimal GoRouter for sign-in flow tests.
Widget _buildApp({
  required AuthBloc authBloc,
  required AuthRepository authRepository,
}) {
  final router = GoRouter(
    initialLocation: AppRoutes.signIn,
    routes: [
      GoRoute(
        path: AppRoutes.signIn,
        builder: (context, _) => MultiBlocProvider(
          providers: [
            BlocProvider<AuthBloc>.value(value: authBloc),
            BlocProvider(
              create: (_) =>
                  SignInCubit(authRepository: authRepository),
            ),
          ],
          child: const SignInView(),
        ),
      ),
      GoRoute(
        path: AppRoutes.signUp,
        builder: (context, _) => BlocProvider(
          create: (_) => SignUpCubit(authRepository: authRepository),
          child: const SignUpView(),
        ),
      ),
      GoRoute(
        path: AppRoutes.forgotPassword,
        builder: (_, _) => const ForgotPasswordView(),
      ),
      GoRoute(
        path: AppRoutes.home,
        builder: (_, _) => BlocProvider<AuthBloc>.value(
          value: authBloc,
          child: const HomeView(),
        ),
      ),
    ],
  );

  return MaterialApp.router(routerConfig: router);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  late MockAuthBloc mockAuthBloc;
  late MockAuthRepository mockAuthRepository;

  setUpAll(() {
    registerFallbackValue(_testUser);
  });

  setUp(() {
    mockAuthBloc = MockAuthBloc();
    mockAuthRepository = MockAuthRepository();
    when(() => mockAuthBloc.state).thenReturn(const AuthUnauthenticated());
  });

  group('Sign-in flow', () {
    testWidgets('SignInView renders form fields and buttons', (tester) async {
      await tester.pumpWidget(
          _buildApp(authBloc: mockAuthBloc, authRepository: mockAuthRepository));
      await tester.pumpAndSettle();

      expect(find.text('Sign In'), findsWidgets);
      expect(find.byType(TextFormField), findsWidgets);
      expect(find.text('Forgot Password?'), findsOneWidget);
      expect(find.text("Don't have an account?"), findsOneWidget);
    });

    testWidgets('successful sign-in updates AuthBloc and navigates to home',
        (tester) async {
      when(() => mockAuthRepository.signIn(
            email: any(named: 'email'),
            password: any(named: 'password'),
          )).thenAnswer(
        (_) async => SignedIn(token: 'tok', user: _testUser),
      );

      await tester.pumpWidget(
          _buildApp(authBloc: mockAuthBloc, authRepository: mockAuthRepository));
      await tester.pumpAndSettle();

      await tester.enterText(
          find.widgetWithText(TextFormField, 'Email'), 'test@example.com');
      await tester.enterText(
          find.widgetWithText(TextFormField, 'Password'), 'password123');

      await tester.tap(
        find.descendant(
            of: find.byType(ElevatedButton), matching: find.text('Sign In')),
      );
      await tester.pumpAndSettle();

      // AuthBloc should receive AuthUserChanged from the listener.
      verify(() => mockAuthBloc.add(any(that: isA<AuthUserChanged>()))).called(1);
    });

    testWidgets('tapping Sign Up navigates to sign-up screen', (tester) async {
      await tester.pumpWidget(
          _buildApp(authBloc: mockAuthBloc, authRepository: mockAuthRepository));
      await tester.pumpAndSettle();

      await tester.tap(find.text('Sign Up'));
      await tester.pumpAndSettle();

      expect(find.text('Sign Up'), findsWidgets);
      expect(find.byType(SignUpView), findsOneWidget);
    });

    testWidgets('tapping Forgot Password navigates to forgot-password screen',
        (tester) async {
      await tester.pumpWidget(
          _buildApp(authBloc: mockAuthBloc, authRepository: mockAuthRepository));
      await tester.pumpAndSettle();

      await tester.tap(find.text('Forgot Password?'));
      await tester.pumpAndSettle();

      expect(find.byType(ForgotPasswordView), findsOneWidget);
    });

    testWidgets('HomeView displays user info from AuthBloc', (tester) async {
      when(() => mockAuthBloc.state)
          .thenReturn(AuthAuthenticated(user: _testUser, token: 'tok'));

      final router = GoRouter(
        initialLocation: AppRoutes.home,
        routes: [
          GoRoute(
            path: AppRoutes.home,
            builder: (_, _) => BlocProvider<AuthBloc>.value(
              value: mockAuthBloc,
              child: const HomeView(),
            ),
          ),
        ],
      );

      await tester.pumpWidget(MaterialApp.router(routerConfig: router));
      await tester.pumpAndSettle();

      expect(find.textContaining('Welcome,'), findsOneWidget);
      expect(find.text('Profile'), findsOneWidget);
      expect(find.text('Settings'), findsOneWidget);
    });
  });
}
