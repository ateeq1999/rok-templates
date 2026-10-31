import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:mobile/app/modules/auth/auth_bloc.dart';
import 'package:mobile/app/modules/auth/sign_in/sign_in_cubit.dart';
import 'package:mobile/app/modules/auth/two_factor/two_factor_cubit.dart';
import 'package:mobile/app/data/repositories/auth.repository.dart';
import 'package:mobile/app/services/auth_service.dart';
import 'package:mobile/app/routes/app_routes.dart';

import '../support/api.dart';

class MockAuthService extends Mock implements AuthService {
  @override
  Stream<AuthStatus> get status => const Stream.empty();
}

class MockAuthRepository extends Mock implements AuthRepository {}

final _testUser = testUser();

void main() {
  setUpAll(() {
    registerFallbackValue(_testUser);
  });

  group('AuthBloc state management', () {
    late MockAuthService mockAuthService;

    setUp(() {
      mockAuthService = MockAuthService();
    });

    blocTest<AuthBloc, AuthState>(
      'initial state is AuthInitial',
      build: () => AuthBloc(authService: mockAuthService),
      verify: (bloc) => expect(bloc.state, isA<AuthInitial>()),
    );

    blocTest<AuthBloc, AuthState>(
      'AuthStarted emits Loading then Unauthenticated when no token stored',
      build: () => AuthBloc(authService: mockAuthService),
      setUp: () {
        when(() => mockAuthService.getToken()).thenAnswer((_) async => null);
        when(() => mockAuthService.getUser()).thenAnswer((_) async => null);
      },
      act: (bloc) => bloc.add(const AuthStarted()),
      expect: () => [isA<AuthLoading>(), isA<AuthUnauthenticated>()],
    );

    blocTest<AuthBloc, AuthState>(
      'AuthStarted emits Loading then Authenticated when token + user stored',
      build: () => AuthBloc(authService: mockAuthService),
      setUp: () {
        when(() => mockAuthService.getToken())
            .thenAnswer((_) async => 'stored-token');
        when(() => mockAuthService.getUser())
            .thenAnswer((_) async => _testUser);
      },
      act: (bloc) => bloc.add(const AuthStarted()),
      expect: () => [isA<AuthLoading>(), isA<AuthAuthenticated>()],
    );

    blocTest<AuthBloc, AuthState>(
      'AuthSignedOut always emits AuthUnauthenticated',
      build: () => AuthBloc(authService: mockAuthService),
      seed: () => AuthAuthenticated(user: _testUser, token: 'tok'),
      act: (bloc) => bloc.add(const AuthSignedOut()),
      expect: () => [isA<AuthUnauthenticated>()],
    );
  });

  group('SignInCubit', () {
    late MockAuthRepository mockRepository;

    setUp(() {
      mockRepository = MockAuthRepository();
    });

    blocTest<SignInCubit, SignInState>(
      'initial state is SignInInitial',
      build: () => SignInCubit(authRepository: mockRepository),
      verify: (cubit) => expect(cubit.state, isA<SignInInitial>()),
    );

    blocTest<SignInCubit, SignInState>(
      'emits Loading then Success on valid credentials',
      build: () => SignInCubit(authRepository: mockRepository),
      setUp: () {
        when(() => mockRepository.signIn(
              email: any(named: 'email'),
              password: any(named: 'password'),
            )).thenAnswer(
          (_) async =>
              SignedIn(token: 'new-token', user: _testUser),
        );
      },
      act: (cubit) =>
          cubit.signIn(email: 'test@example.com', password: 'password123'),
      expect: () => [isA<SignInLoading>(), isA<SignInSuccess>()],
      verify: (cubit) {
        final state = cubit.state as SignInSuccess;
        expect(state.token, equals('new-token'));
        expect(state.user.email, equals('ada@example.com'));
      },
    );

    blocTest<SignInCubit, SignInState>(
      'emits Loading then Failure when repository throws',
      build: () => SignInCubit(authRepository: mockRepository),
      setUp: () {
        when(() => mockRepository.signIn(
              email: any(named: 'email'),
              password: any(named: 'password'),
            )).thenThrow(Exception('Network error'));
      },
      act: (cubit) =>
          cubit.signIn(email: 'bad@example.com', password: 'wrongpass'),
      expect: () => [isA<SignInLoading>(), isA<SignInFailure>()],
    );

    blocTest<SignInCubit, SignInState>(
      'does not emit if already loading (BUG-16 guard)',
      build: () => SignInCubit(authRepository: mockRepository),
      seed: () => const SignInLoading(),
      act: (cubit) =>
          cubit.signIn(email: 'test@example.com', password: 'password'),
      expect: () => <SignInState>[],
    );

    blocTest<SignInCubit, SignInState>(
      'asks for the second step when the account has two-factor on',
      build: () => SignInCubit(authRepository: mockRepository),
      setUp: () {
        when(() => mockRepository.signIn(
              email: any(named: 'email'),
              password: any(named: 'password'),
            )).thenAnswer((_) async => const TwoFactorRequired('pending-1'));
      },
      act: (cubit) =>
          cubit.signIn(email: 'ada@example.com', password: 'password123'),
      expect: () => [
        isA<SignInLoading>(),
        isA<SignInTwoFactorRequired>()
            .having((s) => s.pendingToken, 'pendingToken', 'pending-1'),
      ],
    );
  });

  group('TwoFactorCubit', () {
    late MockAuthRepository mockRepository;

    setUp(() => mockRepository = MockAuthRepository());

    blocTest<TwoFactorCubit, TwoFactorState>(
      'verifies the code with the pending token and signs in',
      build: () => TwoFactorCubit(
        authRepository: mockRepository,
        pendingToken: 'pending-1',
      ),
      setUp: () {
        when(() => mockRepository.verifyTwoFactor(
              pendingToken: 'pending-1',
              code: '123456',
            )).thenAnswer(
          (_) async => SignedIn(user: _testUser, token: 'tok-2'),
        );
      },
      act: (cubit) => cubit.verify(code: '123456'),
      expect: () => [
        isA<TwoFactorLoading>(),
        isA<TwoFactorSuccess>().having((s) => s.token, 'token', 'tok-2'),
      ],
    );
  });

  group('Route constants', () {
    test('all required routes are defined', () {
      expect(AppRoutes.signIn, equals('/sign-in'));
      expect(AppRoutes.signUp, equals('/sign-up'));
      expect(AppRoutes.home, equals('/home'));
      expect(AppRoutes.splash, equals('/splash'));
    });
  });
}
