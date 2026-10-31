import 'package:bloc_test/bloc_test.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:mobile/app/data/models/admin_user.model.dart';
import 'package:mobile/app/data/providers/admin.provider.dart';
import 'package:mobile/app/data/repositories/admin.repository.dart';
import 'package:mobile/app/modules/admin/admin_users_cubit.dart';

class MockAdminProvider extends Mock implements AdminProvider {}

class MockAdminRepository extends Mock implements AdminRepository {}

class _FakeCancelToken extends Fake implements CancelToken {}

Response<dynamic> _response(dynamic data) => Response<dynamic>(
      requestOptions: RequestOptions(path: '/v1/api/admin/users'),
      statusCode: 200,
      data: data,
    );

Map<String, dynamic> _userJson(String id, {String email = 'a@b.com'}) => {
      'id': id,
      'email': email,
      'name': 'Name $id',
      'role': 'user',
      'emailVerified': true,
      'banned': false,
      'deleted': false,
      'createdAt': '2024-01-01T00:00:00.000Z',
    };

Map<String, dynamic> _page({
  required List<Map<String, dynamic>> users,
  String? nextCursor,
  bool hasNextPage = false,
}) => {
      'success': true,
      'data': users,
      'meta': {
        'timestamp': '2024-01-01T00:00:00.000Z',
        'requestId': 'req-1',
        'pagination': {
          'limit': 20,
          'hasNextPage': hasNextPage,
          'nextCursor': nextCursor,
        },
      },
    };

final _fallbackAdminUser = AdminUser(
  id: 'f',
  email: 'f@b.com',
  emailVerified: true,
  banned: false,
  deleted: false,
  createdAt: DateTime(2024),
);

void main() {
  setUpAll(() {
    registerFallbackValue(_fallbackAdminUser);
    registerFallbackValue(_FakeCancelToken());
  });

  group('AdminRepository.listUsers — pagination', () {
    late MockAdminProvider provider;
    late AdminRepository repo;

    setUp(() {
      provider = MockAdminProvider();
      repo = AdminRepository(provider);
    });

    test('reads nextCursor + hasNextPage from meta.pagination', () async {
      when(() => provider.listUsers(
            limit: any(named: 'limit'),
            cursor: any(named: 'cursor'),
            search: any(named: 'search'),
            role: any(named: 'role'),
            verified: any(named: 'verified'),
            cancelToken: any(named: 'cancelToken'),
          )).thenAnswer((_) async => _response(_page(
            users: [_userJson('1'), _userJson('2')],
            nextCursor: 'cur-xyz',
            hasNextPage: true,
          )));

      final result = await repo.listUsers();

      expect(result.users, hasLength(2));
      expect(result.cursor, 'cur-xyz');
      expect(result.hasMore, isTrue);
    });

    test('treats missing pagination block as last page', () async {
      when(() => provider.listUsers(
            limit: any(named: 'limit'),
            cursor: any(named: 'cursor'),
            search: any(named: 'search'),
            role: any(named: 'role'),
            verified: any(named: 'verified'),
            cancelToken: any(named: 'cancelToken'),
          )).thenAnswer((_) async => _response({
            'success': true,
            'data': [_userJson('1')],
            'meta': {
              'timestamp': '2024-01-01T00:00:00.000Z',
              'requestId': 'r',
            },
          }));

      final result = await repo.listUsers();

      expect(result.cursor, isNull);
      expect(result.hasMore, isFalse);
    });
  });

  group('AdminUsersCubit — pagination', () {
    late MockAdminRepository repo;

    setUp(() {
      repo = MockAdminRepository();
    });

    blocTest<AdminUsersCubit, AdminUsersState>(
      'loadUsers emits Loading then Loaded with first page',
      build: () => AdminUsersCubit(adminRepository: repo),
      setUp: () {
        when(() => repo.listUsers(
              limit: any(named: 'limit'),
              cursor: any(named: 'cursor'),
              search: any(named: 'search'),
              role: any(named: 'role'),
              verified: any(named: 'verified'),
              cancelToken: any(named: 'cancelToken'),
            )).thenAnswer((_) async => (
              users: [AdminUser.fromJson(_userJson('1'))],
              cursor: 'cur-1',
              hasMore: true,
            ));
      },
      act: (c) => c.loadUsers(),
      expect: () => [
        isA<AdminUsersLoading>(),
        isA<AdminUsersLoaded>()
            .having((s) => s.users.length, 'users.length', 1)
            .having((s) => s.hasMore, 'hasMore', true)
            .having((s) => s.cursor, 'cursor', 'cur-1'),
      ],
    );

    blocTest<AdminUsersCubit, AdminUsersState>(
      'loadMore appends next page and advances cursor',
      build: () => AdminUsersCubit(adminRepository: repo),
      seed: () => AdminUsersLoaded(
        users: [AdminUser.fromJson(_userJson('1'))],
        cursor: 'cur-1',
        hasMore: true,
        search: '',
      ),
      setUp: () {
        when(() => repo.listUsers(
              limit: any(named: 'limit'),
              cursor: any(named: 'cursor'),
              search: any(named: 'search'),
              role: any(named: 'role'),
              verified: any(named: 'verified'),
              cancelToken: any(named: 'cancelToken'),
            )).thenAnswer((_) async => (
              users: [AdminUser.fromJson(_userJson('2'))],
              cursor: null,
              hasMore: false,
            ));
      },
      act: (c) => c.loadMore(),
      expect: () => [
        isA<AdminUsersLoaded>()
            .having((s) => s.users.map((u) => u.id).toList(), 'ids',
                ['1', '2'])
            .having((s) => s.hasMore, 'hasMore', false)
            .having((s) => s.cursor, 'cursor', isNull),
      ],
    );

    blocTest<AdminUsersCubit, AdminUsersState>(
      'loadMore is no-op when hasMore is false',
      build: () => AdminUsersCubit(adminRepository: repo),
      seed: () => AdminUsersLoaded(
        users: [AdminUser.fromJson(_userJson('1'))],
        cursor: null,
        hasMore: false,
        search: '',
      ),
      act: (c) => c.loadMore(),
      expect: () => <AdminUsersState>[],
      verify: (_) {
        verifyNever(() => repo.listUsers(
              limit: any(named: 'limit'),
              cursor: any(named: 'cursor'),
              search: any(named: 'search'),
              role: any(named: 'role'),
              verified: any(named: 'verified'),
            ));
      },
    );
  });
}
