import 'package:bloc_test/bloc_test.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:mobile/app/data/models/organization.model.dart';
import 'package:mobile/app/data/providers/organization.provider.dart';
import 'package:mobile/app/data/repositories/organization.repository.dart';
import 'package:mobile/app/modules/organizations/org_list_cubit.dart';

class MockOrgProvider extends Mock implements OrganizationProvider {}

class MockOrgRepository extends Mock implements OrganizationRepository {}

class _FakeCancelToken extends Fake implements CancelToken {}

Response<dynamic> _response(dynamic data) => Response<dynamic>(
      requestOptions: RequestOptions(path: '/v1/api/orgs'),
      statusCode: 200,
      data: data,
    );

Map<String, dynamic> _orgJson(String id, {String name = 'Org'}) => {
      'id': id,
      'name': '$name $id',
      'createdAt': '2024-01-01T00:00:00.000Z',
    };

Map<String, dynamic> _listEnvelope(List<Map<String, dynamic>> items) => {
      'success': true,
      'data': items,
      'meta': {'timestamp': '2024-01-01T00:00:00.000Z', 'requestId': 'r'},
    };

Map<String, dynamic> _itemEnvelope(Map<String, dynamic> data) => {
      'success': true,
      'data': data,
      'meta': {'timestamp': '2024-01-01T00:00:00.000Z', 'requestId': 'r'},
    };

void main() {
  setUpAll(() {
    registerFallbackValue(_FakeCancelToken());
  });

  group('OrganizationRepository', () {
    late MockOrgProvider provider;
    late OrganizationRepository repo;

    setUp(() {
      provider = MockOrgProvider();
      repo = OrganizationRepository(provider);
    });

    test('listOrgs unwraps list envelope', () async {
      when(() => provider.listOrgs(
            cancelToken: any(named: 'cancelToken'),
          )).thenAnswer(
        (_) async =>
            _response(_listEnvelope([_orgJson('1'), _orgJson('2')])),
      );

      final orgs = await repo.listOrgs();

      expect(orgs, hasLength(2));
      expect(orgs.first.id, '1');
      expect(orgs.last.id, '2');
    });

    test('createOrg unwraps single envelope', () async {
      when(() => provider.createOrg(any())).thenAnswer(
        (_) async => _response(_itemEnvelope(_orgJson('new'))),
      );

      final org = await repo.createOrg('new');

      expect(org.id, 'new');
    });
  });

  group('OrgListCubit', () {
    late MockOrgRepository repo;

    setUp(() {
      repo = MockOrgRepository();
    });

    blocTest<OrgListCubit, OrgListState>(
      'loadOrgs emits Loading then Loaded',
      build: () => OrgListCubit(orgRepository: repo),
      setUp: () {
        when(() => repo.listOrgs(
              cancelToken: any(named: 'cancelToken'),
            )).thenAnswer((_) async => [
              Organization.fromJson(_orgJson('1')),
              Organization.fromJson(_orgJson('2')),
            ]);
      },
      act: (c) => c.loadOrgs(),
      expect: () => [
        isA<OrgListLoading>(),
        isA<OrgListLoaded>()
            .having((s) => s.orgs.map((o) => o.id).toList(), 'ids', ['1', '2']),
      ],
    );

    blocTest<OrgListCubit, OrgListState>(
      'createOrg appends to existing Loaded state',
      build: () => OrgListCubit(orgRepository: repo),
      seed: () => OrgListLoaded([Organization.fromJson(_orgJson('1'))]),
      setUp: () {
        when(() => repo.createOrg(any())).thenAnswer(
          (_) async => Organization.fromJson(_orgJson('2')),
        );
      },
      act: (c) => c.createOrg('New Org'),
      expect: () => [
        isA<OrgListLoaded>().having(
            (s) => s.orgs.map((o) => o.id).toList(), 'ids', ['1', '2']),
      ],
    );

    blocTest<OrgListCubit, OrgListState>(
      'deleteOrg removes from list',
      build: () => OrgListCubit(orgRepository: repo),
      seed: () => OrgListLoaded([
        Organization.fromJson(_orgJson('1')),
        Organization.fromJson(_orgJson('2')),
      ]),
      setUp: () {
        when(() => repo.deleteOrg(any())).thenAnswer((_) async {});
      },
      act: (c) => c.deleteOrg('1'),
      expect: () => [
        isA<OrgListLoaded>()
            .having((s) => s.orgs.map((o) => o.id).toList(), 'ids', ['2']),
      ],
    );
  });
}
