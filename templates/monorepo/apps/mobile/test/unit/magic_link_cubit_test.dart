import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:mobile/app/core/errors/app_exception.dart';
import 'package:mobile/app/data/models/auth_response.model.dart';
import 'package:mobile/app/data/models/user.model.dart';
import 'package:mobile/app/data/repositories/auth.repository.dart';
import 'package:mobile/app/modules/auth/magic_link/magic_link_cubit.dart';

class MockAuthRepository extends Mock implements AuthRepository {}

final _testUser = User(
  id: 'u-1',
  email: 'a@b.com',
  name: 'Alice',
  emailVerified: true,
  createdAt: DateTime(2024),
);

void main() {
  late MockAuthRepository repo;

  setUp(() {
    repo = MockAuthRepository();
  });

  group('MagicLinkCubit.sendLink', () {
    blocTest<MagicLinkCubit, MagicLinkState>(
      'emits Loading then Sent on success',
      build: () => MagicLinkCubit(authRepository: repo),
      setUp: () {
        when(() => repo.sendMagicLink(email: any(named: 'email')))
            .thenAnswer((_) async {});
      },
      act: (c) => c.sendLink(email: 'a@b.com'),
      expect: () => [
        isA<MagicLinkLoading>(),
        isA<MagicLinkSent>().having((s) => s.email, 'email', 'a@b.com'),
      ],
    );

    blocTest<MagicLinkCubit, MagicLinkState>(
      'emits Loading then Failure on ApiException',
      build: () => MagicLinkCubit(authRepository: repo),
      setUp: () {
        when(() => repo.sendMagicLink(email: any(named: 'email')))
            .thenThrow(ApiException(message: 'Rate limited', statusCode: 429));
      },
      act: (c) => c.sendLink(email: 'a@b.com'),
      expect: () => [
        isA<MagicLinkLoading>(),
        isA<MagicLinkFailure>().having((s) => s.message, 'message', 'Rate limited'),
      ],
    );

    blocTest<MagicLinkCubit, MagicLinkState>(
      'does not emit if already loading',
      build: () => MagicLinkCubit(authRepository: repo),
      seed: () => const MagicLinkLoading(),
      act: (c) => c.sendLink(email: 'a@b.com'),
      expect: () => <MagicLinkState>[],
    );
  });

  group('MagicLinkCubit.verifyLink', () {
    blocTest<MagicLinkCubit, MagicLinkState>(
      'emits Loading then Verified with token + user on success',
      build: () => MagicLinkCubit(authRepository: repo),
      setUp: () {
        when(() => repo.verifyMagicLink(token: any(named: 'token')))
            .thenAnswer((_) async =>
                AuthResponse(token: 'bat_abc', user: _testUser));
      },
      act: (c) => c.verifyLink(token: 'tok-1'),
      expect: () => [
        isA<MagicLinkLoading>(),
        isA<MagicLinkVerified>()
            .having((s) => s.token, 'token', 'bat_abc')
            .having((s) => s.user.email, 'user.email', 'a@b.com'),
      ],
    );

    blocTest<MagicLinkCubit, MagicLinkState>(
      'is a no-op for empty token',
      build: () => MagicLinkCubit(authRepository: repo),
      act: (c) => c.verifyLink(token: ''),
      expect: () => <MagicLinkState>[],
      verify: (_) {
        verifyNever(() => repo.verifyMagicLink(token: any(named: 'token')));
      },
    );

    blocTest<MagicLinkCubit, MagicLinkState>(
      'emits Failure when token is invalid',
      build: () => MagicLinkCubit(authRepository: repo),
      setUp: () {
        when(() => repo.verifyMagicLink(token: any(named: 'token')))
            .thenThrow(ApiException(message: 'Expired link', statusCode: 400));
      },
      act: (c) => c.verifyLink(token: 'bad'),
      expect: () => [
        isA<MagicLinkLoading>(),
        isA<MagicLinkFailure>().having((s) => s.message, 'message', 'Expired link'),
      ],
    );
  });
}
