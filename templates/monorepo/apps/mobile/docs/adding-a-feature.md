# Adding a Feature

End to end: an API call, a repository method, a cubit, a view and a route.
The worked example lists the signed-in user's files (`GET /api/v1/media`,
which `apps/api` serves) on a "My files" screen.

## Checklist

1. The API serves it, and the client has it (`rok gen`)
2. Repository method
3. Cubit + state
4. View
5. Route (and a link to it)
6. Feature flag (optional)
7. Tests

## 1. The API and the client

The App calls only what `apps/api`'s contract has. A new endpoint is
written in `apps/api` (with its `utoipa` annotation); then, from the
monorepo's root:

```bash
rok gen
```

It re-exports `apps/api/openapi.json` and regenerates `clients/dart`: the
operation appears as `api.<tag>.<operation>(...)`, with its models. For the
example, it is already there:

```dart
Future<PaginatedResponseMediaResponse> list({int? page, int? perPage, RequestOptions? options})
```

Never edit `clients/dart` by hand, and never call the API around it.

## 2. Repository

`lib/app/data/repositories/media.repository.dart` — what the App does with
the calls. No HTTP, no JSON: the client's models in, the client's models out.

```dart
import 'package:api_client/api_client.dart';

class MediaRepository {
  MediaRepository(this._api);

  final ApiClient _api;

  Future<PaginatedResponseMediaResponse> myFiles({int page = 1}) =>
      _api.media.list(page: page, perPage: 20);
}
```

Every failure is the client's `ApiException`; let it through.

## 3. Cubit + state

`lib/app/modules/media/media_cubit.dart`:

```dart
import 'package:api_client/api_client.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../data/repositories/media.repository.dart';

part 'media_state.dart';

class MediaCubit extends Cubit<MediaState> {
  MediaCubit({required MediaRepository mediaRepository})
      : _mediaRepository = mediaRepository,
        super(const MediaInitial());

  final MediaRepository _mediaRepository;

  Future<void> load() async {
    if (state is MediaLoading) return;
    emit(const MediaLoading());
    try {
      final page = await _mediaRepository.myFiles();
      emit(MediaLoaded(page.items));
    } on ApiException catch (e) {
      emit(MediaFailure(e.message));
    }
  }
}
```

`media_state.dart`:

```dart
part of 'media_cubit.dart';

sealed class MediaState {
  const MediaState();
}

final class MediaInitial extends MediaState {
  const MediaInitial();
}

final class MediaLoading extends MediaState {
  const MediaLoading();
}

final class MediaLoaded extends MediaState {
  const MediaLoaded(this.files);
  final List<MediaResponse> files;
}

final class MediaFailure extends MediaState {
  const MediaFailure(this.message);
  final String message;
}
```

A form's failure state also carries `e.fieldErrors` (import
`core/errors/field_errors.dart`): one message per field the API rejected,
for `TextFormField.forceErrorText`.

## 4. View

`lib/app/modules/media/media_view.dart`:

```dart
class MediaView extends StatelessWidget {
  const MediaView({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('My files')),
      body: BlocBuilder<MediaCubit, MediaState>(
        builder: (context, state) => switch (state) {
          MediaInitial() || MediaLoading() =>
            const Center(child: CircularProgressIndicator()),
          MediaFailure(:final message) => Center(child: Text(message)),
          MediaLoaded(:final files) => ListView(
              children: [
                for (final file in files)
                  ListTile(
                    title: Text(file.originalFilename ?? file.id),
                    subtitle: Text('${file.sizeBytes} bytes'),
                  ),
              ],
            ),
        },
      ),
    );
  }
}
```

## 5. Route

`lib/app/routes/app_routes.dart`: `static const media = '/media';`

In [app_router.dart](../lib/app/core/router/app_router.dart), with the
protected routes:

```dart
GoRoute(
  path: AppRoutes.media,
  builder: (context, _) => BlocProvider(
    create: (_) => MediaCubit(
      mediaRepository: context.read<MediaRepository>(),
    )..load(),
    child: const MediaView(),
  ),
),
```

In `main.dart`, build the repository beside the others and provide it:

```dart
final mediaRepository = MediaRepository(api);
// ...
RepositoryProvider<MediaRepository>.value(value: mediaRepository),
```

Then link to it, say from [home_view.dart](../lib/app/modules/home/home_view.dart):

```dart
ListTile(
  leading: const Icon(Icons.folder),
  title: const Text('My files'),
  onTap: () => context.push(AppRoutes.media),
),
```

## 6. Feature flag (optional)

See [feature-flags.md](./feature-flags.md#adding-a-new-flag).

## 7. Test

The repository against the real client over a fake API
([test/support/api.dart](../test/support/api.dart)):

```dart
test('myFiles reads the first page', () async {
  final requests = <http.Request>[];
  final repo = MediaRepository(fakeApi(
    (_) async => json({'items': [], 'page': 1, 'per_page': 20, 'total': 0}),
    session: Session(token: () => 'tok-1'),
    requests: requests,
  ));

  final page = await repo.myFiles();

  expect(requests.single.url.queryParameters, {'page': '1', 'per_page': '20'});
  expect(page.total, 0);
});
```

The cubit with `bloc_test` and a `mocktail` repository, as
[test/unit/auth_service_test.dart](../test/unit/auth_service_test.dart)
does for `SignInCubit`.

```bash
flutter analyze
flutter test
```

## Common pitfalls

- **Writing an HTTP call by hand.** The API's operation is missing from
  the client: add it to `apps/api` and run `rok gen`.
- **A `401` that is not an expired session** (a wrong code or password on
  a secured call): pass `options: const RequestOptions(keepSession: true)`,
  or the client signs out.
- **Forgetting the provider.** A repository read with `context.read` in a
  route must be in `main.dart`'s `MultiRepositoryProvider`.
