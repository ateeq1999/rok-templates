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
}) {
  return RepositoryProvider<FeatureFlags>.value(
    value: _testFlags,
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
    when(() => mockAuthBloc.state).thenReturn(const AuthUnauthenticated());
  });

  group('SignInView — loading indicator', () {
    testWidgets('shows CircularProgressIndicator when state is SignInLoading',
        (tester) async {
      when(() => mockCubit.state).thenReturn(const SignInLoading());

      await tester
          .pumpWidget(_buildSut(cubit: mockCubit, authBloc: mockAuthBloc));
      await tester.pump();

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(
        find.descendant(
          of: find.byType(ElevatedButton),
          matching: find.text('Sign In'),
        ),
        findsNothing,
      );
    });

    testWidgets('shows Sign In text when state is SignInInitial',
        (tester) async {
      when(() => mockCubit.state).thenReturn(const SignInInitial());

      await tester
          .pumpWidget(_buildSut(cubit: mockCubit, authBloc: mockAuthBloc));
      await tester.pump();

      expect(
        find.descendant(
          of: find.byType(ElevatedButton),
          matching: find.text('Sign In'),
        ),
        findsOneWidget,
      );
      expect(find.byType(CircularProgressIndicator), findsNothing);
    });

    testWidgets('ElevatedButton.onPressed is null during loading',
        (tester) async {
      when(() => mockCubit.state).thenReturn(const SignInLoading());

      await tester
          .pumpWidget(_buildSut(cubit: mockCubit, authBloc: mockAuthBloc));
      await tester.pump();

      final button =
          tester.widget<ElevatedButton>(find.byType(ElevatedButton));
      expect(button.onPressed, isNull);
    });

    testWidgets('ElevatedButton.onPressed is not null when not loading',
        (tester) async {
      when(() => mockCubit.state).thenReturn(const SignInInitial());

      await tester
          .pumpWidget(_buildSut(cubit: mockCubit, authBloc: mockAuthBloc));
      await tester.pump();

      final button =
          tester.widget<ElevatedButton>(find.byType(ElevatedButton));
      expect(button.onPressed, isNotNull);
    });

    testWidgets('shows no progress indicator when in failure state',
        (tester) async {
      when(() => mockCubit.state)
          .thenReturn(const SignInFailure('Invalid credentials'));

      await tester
          .pumpWidget(_buildSut(cubit: mockCubit, authBloc: mockAuthBloc));
      await tester.pump();

      expect(find.byType(CircularProgressIndicator), findsNothing);
      expect(
        find.descendant(
          of: find.byType(ElevatedButton),
          matching: find.text('Sign In'),
        ),
        findsOneWidget,
      );
    });
  });
}
