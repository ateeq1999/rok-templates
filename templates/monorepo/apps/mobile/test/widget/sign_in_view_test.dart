import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:go_router/go_router.dart';

import 'package:mobile/app/core/config/feature_flags.dart';
import 'package:mobile/app/modules/auth/auth_bloc.dart';
import 'package:mobile/app/modules/auth/sign_in/sign_in_cubit.dart';
import 'package:mobile/app/modules/auth/sign_in/sign_in_view.dart';
import 'package:mobile/app/services/auth_service.dart';

class MockSignInCubit extends MockCubit<SignInState> implements SignInCubit {
  @override
  bool get isPasswordHidden => true;
}

class MockAuthBloc extends MockBloc<AuthEvent, AuthState> implements AuthBloc {}

class MockAuthService extends Mock implements AuthService {
  @override
  Stream<AuthStatus> get status => const Stream.empty();
}

const _testFlags = FeatureFlags(
  magicLink: true,
  twoFactor: true,
  organizations: true,
  admin: true,
  signUp: true,
  oauth: false,
  notifications: true,
  themeCustomization: true,
  deleteAccount: false,
);

Widget _buildSut({
  required SignInCubit cubit,
  required AuthBloc authBloc,
  FeatureFlags flags = _testFlags,
}) {
  return RepositoryProvider<FeatureFlags>.value(
    value: flags,
    child: MaterialApp.router(
      routerConfig: GoRouter(
        initialLocation: '/sign-in',
        routes: [
          GoRoute(
            path: '/sign-in',
            builder: (ctx, _) => MultiBlocProvider(
              providers: [
                BlocProvider<AuthBloc>.value(value: authBloc),
                BlocProvider<SignInCubit>.value(value: cubit),
              ],
              child: const SignInView(),
            ),
          ),
          GoRoute(path: '/sign-up', builder: (_, _) => const Scaffold()),
          GoRoute(
              path: '/forgot-password', builder: (_, _) => const Scaffold()),
          GoRoute(path: '/home', builder: (_, _) => const Scaffold()),
        ],
      ),
    ),
  );
}

void main() {
  late MockSignInCubit mockCubit;
  late MockAuthBloc mockAuthBloc;

  setUp(() {
    mockCubit = MockSignInCubit();
    mockAuthBloc = MockAuthBloc();
    when(() => mockCubit.state).thenReturn(const SignInInitial());
    when(() => mockAuthBloc.state).thenReturn(const AuthUnauthenticated());
  });

  group('SignInView — form field rendering', () {
    testWidgets('displays email TextFormField', (tester) async {
      await tester.pumpWidget(
          _buildSut(cubit: mockCubit, authBloc: mockAuthBloc));
      await tester.pumpAndSettle();

      expect(find.widgetWithText(TextFormField, 'Email'), findsOneWidget);
    });

    testWidgets('displays password TextFormField', (tester) async {
      await tester.pumpWidget(
          _buildSut(cubit: mockCubit, authBloc: mockAuthBloc));
      await tester.pumpAndSettle();

      expect(find.widgetWithText(TextFormField, 'Password'), findsOneWidget);
    });

    testWidgets('displays Sign In ElevatedButton', (tester) async {
      await tester.pumpWidget(
          _buildSut(cubit: mockCubit, authBloc: mockAuthBloc));
      await tester.pumpAndSettle();

      expect(
        find.descendant(
          of: find.byType(ElevatedButton),
          matching: find.text('Sign In'),
        ),
        findsOneWidget,
      );
    });

    testWidgets('displays Forgot Password link', (tester) async {
      await tester.pumpWidget(
          _buildSut(cubit: mockCubit, authBloc: mockAuthBloc));
      await tester.pumpAndSettle();

      expect(find.text('Forgot Password?'), findsOneWidget);
    });

    testWidgets('displays Sign Up link', (tester) async {
      await tester.pumpWidget(
          _buildSut(cubit: mockCubit, authBloc: mockAuthBloc));
      await tester.pumpAndSettle();

      expect(find.text("Don't have an account?"), findsOneWidget);
      expect(find.text('Sign Up'), findsOneWidget);
    });

    testWidgets('displays password visibility toggle icon', (tester) async {
      await tester.pumpWidget(
          _buildSut(cubit: mockCubit, authBloc: mockAuthBloc));
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.visibility_off), findsOneWidget);
    });

    testWidgets('displays AppBar with Sign In title', (tester) async {
      await tester.pumpWidget(
          _buildSut(cubit: mockCubit, authBloc: mockAuthBloc));
      await tester.pumpAndSettle();

      expect(find.byType(AppBar), findsOneWidget);
    });
  });
}
