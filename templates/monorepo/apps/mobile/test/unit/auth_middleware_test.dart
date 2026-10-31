// Previously tested GetX AuthMiddleware.
// Route guards are now implemented as a GoRouter redirect callback that reads
// AuthBloc state. These tests verify the AuthBloc state transitions that drive
// the redirect logic.

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:mobile/app/data/models/user.model.dart';
import 'package:mobile/app/modules/auth/auth_bloc.dart';
import 'package:mobile/app/services/auth_service.dart';
import 'package:mobile/app/routes/app_routes.dart';

class MockAuthService extends Mock implements AuthService {
  @override
  Stream<AuthStatus> get status => const Stream.empty();
}

final _testUser = User(
  id: 'user-1',
  email: 'test@example.com',
  name: 'Test User',
  emailVerified: true,
  createdAt: DateTime(2024),
);

void main() {
  late MockAuthService mockAuthService;

  setUp(() {
    mockAuthService = MockAuthService();
  });

  group('AuthBloc — route guard states', () {
    blocTest<AuthBloc, AuthState>(
      'emits AuthUnauthenticated (→ redirects to ${AppRoutes.signIn}) when no token',
      build: () => AuthBloc(authService: mockAuthService),
      setUp: () {
        when(() => mockAuthService.getToken()).thenAnswer((_) async => null);
        when(() => mockAuthService.getUser()).thenAnswer((_) async => null);
      },
      act: (bloc) => bloc.add(const AuthStarted()),
      expect: () => [isA<AuthLoading>(), isA<AuthUnauthenticated>()],
    );

    blocTest<AuthBloc, AuthState>(
      'emits AuthAuthenticated (→ no redirect) when valid token + user exist',
      build: () => AuthBloc(authService: mockAuthService),
      setUp: () {
        when(() => mockAuthService.getToken())
            .thenAnswer((_) async => 'valid-token');
        when(() => mockAuthService.getUser())
            .thenAnswer((_) async => _testUser);
      },
      act: (bloc) => bloc.add(const AuthStarted()),
      expect: () => [isA<AuthLoading>(), isA<AuthAuthenticated>()],
      verify: (bloc) {
        final state = bloc.state as AuthAuthenticated;
        expect(state.token, equals('valid-token'));
        expect(state.user.email, equals('test@example.com'));
      },
    );

    blocTest<AuthBloc, AuthState>(
      'emits AuthUnauthenticated when AuthSignedOut is added',
      build: () => AuthBloc(authService: mockAuthService),
      seed: () => AuthAuthenticated(user: _testUser, token: 'tok'),
      act: (bloc) => bloc.add(const AuthSignedOut()),
      expect: () => [isA<AuthUnauthenticated>()],
    );

    blocTest<AuthBloc, AuthState>(
      'emits AuthAuthenticated when AuthUserChanged is added',
      build: () => AuthBloc(authService: mockAuthService),
      seed: () => const AuthUnauthenticated(),
      act: (bloc) =>
          bloc.add(AuthUserChanged(user: _testUser, token: 'new-token')),
      expect: () => [isA<AuthAuthenticated>()],
    );
  });

  group('AppRoutes constants', () {
    test('all route paths are defined correctly', () {
      expect(AppRoutes.splash, equals('/splash'));
      expect(AppRoutes.signIn, equals('/sign-in'));
      expect(AppRoutes.signUp, equals('/sign-up'));
      expect(AppRoutes.forgotPassword, equals('/forgot-password'));
      expect(AppRoutes.resetPassword, equals('/reset-password'));
      expect(AppRoutes.verifyEmail, equals('/verify-email'));
      expect(AppRoutes.twoFactor, equals('/two-factor'));
      expect(AppRoutes.home, equals('/home'));
      expect(AppRoutes.profile, equals('/profile'));
      expect(AppRoutes.settings, equals('/settings'));
    });
  });
}
