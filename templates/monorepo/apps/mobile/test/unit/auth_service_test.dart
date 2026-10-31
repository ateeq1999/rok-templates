import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:mobile/app/data/models/user.model.dart';
import 'package:mobile/app/modules/auth/auth_bloc.dart';
import 'package:mobile/app/modules/auth/sign_in/sign_in_cubit.dart';
import 'package:mobile/app/data/repositories/auth.repository.dart';
import 'package:mobile/app/data/models/auth_response.model.dart';
import 'package:mobile/app/services/auth_service.dart';
import 'package:mobile/app/routes/app_routes.dart';

class MockAuthService extends Mock implements AuthService {
  @override
  Stream<AuthStatus> get status => const Stream.empty();
}

class MockAuthRepository extends Mock implements AuthRepository {}

final _testUser = User(
  id: 'user-1',
  email: 'test@example.com',
  name: 'Test User',
  emailVerified: true,
  createdAt: DateTime(2024),
);

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
              AuthResponse(token: 'new-token', user: _testUser),
        );
      },
      act: (cubit) =>
          cubit.signIn(email: 'test@example.com', password: 'password123'),
      expect: () => [isA<SignInLoading>(), isA<SignInSuccess>()],
      verify: (cubit) {
        final state = cubit.state as SignInSuccess;
        expect(state.token, equals('new-token'));
        expect(state.user.email, equals('test@example.com'));
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
