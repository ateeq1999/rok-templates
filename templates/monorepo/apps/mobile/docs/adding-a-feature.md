# Adding a Feature

End-to-end walkthrough: wire a new API call, a cubit, a screen, a route, and a test. We'll use a hypothetical **Notes** feature (`GET /v1/api/notes`, `POST /v1/api/notes`) as the running example.

The same steps apply to any resource — admin users, organizations, whatever. Open [organizations](../lib/app/modules/organizations/) as a reference implementation while you read.

## Checklist

```
□ 1. Model         lib/app/data/models/note.model.dart
□ 2. Provider      lib/app/data/providers/note.provider.dart
□ 3. Repository    lib/app/data/repositories/note.repository.dart
□ 4. Cubit + state lib/app/modules/notes/note_list_cubit.dart
□                  lib/app/modules/notes/note_list_state.dart
□ 5. View          lib/app/modules/notes/notes_view.dart
□ 6. Route         lib/app/routes/app_routes.dart
□                  lib/app/core/router/app_router.dart
□ 7. DI            lib/main.dart
□ 8. Feature flag  lib/app/core/config/feature_flags.dart + .env
□ 9. Test          test/unit/note_repository_test.dart
```

## 1. Model

```dart
// lib/app/data/models/note.model.dart
class Note {
  Note({required this.id, required this.title, required this.createdAt});

  final String id;
  final String title;
  final DateTime createdAt;

  factory Note.fromJson(Map<String, dynamic> json) => Note(
        id: json['id'] as String,
        title: json['title'] as String,
        createdAt: DateTime.parse(json['createdAt'] as String),
      );

  Map<String, dynamic> toJson() => {
        'id': id,
        'title': title,
        'createdAt': createdAt.toIso8601String(),
      };
}
```

Freezed is available for models that need `copyWith` / equality — see [user.model.dart](../lib/app/data/models/user.model.dart). For simple read-only DTOs, plain classes are fine.

## 2. Provider — raw Dio

```dart
// lib/app/data/providers/note.provider.dart
import 'package:dio/dio.dart';

class NoteProvider {
  NoteProvider(this._dio);
  final Dio _dio;

  Future<Response<dynamic>> listNotes({CancelToken? cancelToken}) =>
      _dio.get('/v1/api/notes', cancelToken: cancelToken);

  Future<Response<dynamic>> createNote(String title) =>
      _dio.post('/v1/api/notes', data: {'title': title});
}
```

- All paths start with `/v1/api/`.
- Pass `cancelToken` on list/read endpoints — the cubit will cancel stale requests.
- Don't unwrap the envelope here. The provider is dumb on purpose.

## 3. Repository — business logic

```dart
// lib/app/data/repositories/note.repository.dart
import 'package:dio/dio.dart';

import '../models/note.model.dart';
import '../providers/note.provider.dart';
import '../../core/errors/app_exception.dart';
import '../../core/utils/response_parser.dart';

class NoteRepository {
  NoteRepository(this._provider);
  final NoteProvider _provider;

  Future<List<Note>> listNotes({CancelToken? cancelToken}) async {
    try {
      final response = await _provider.listNotes(cancelToken: cancelToken);
      return unwrapEnvelopeList(response.data).map(Note.fromJson).toList();
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }

  Future<Note> createNote(String title) async {
    try {
      final response = await _provider.createNote(title);
      return Note.fromJson(unwrapEnvelope(response.data));
    } on DioException catch (e) {
      throw ApiException.fromDioError(e);
    }
  }
}
```

Three rules:

1. Wrap every provider call in `try/on DioException`.
2. Unwrap with `unwrapEnvelope` / `unwrapEnvelopeList` from [response_parser.dart](../lib/app/core/utils/response_parser.dart).
3. Throw `ApiException.fromDioError(e)` — never leak `DioException` to the UI layer.

## 4. Cubit + state

**Sealed states** (see [org_list_state.dart](../lib/app/modules/organizations/org_list_state.dart) for the canonical shape):

```dart
// lib/app/modules/notes/note_list_state.dart
part of 'note_list_cubit.dart';

sealed class NoteListState { const NoteListState(); }
final class NoteListInitial extends NoteListState { const NoteListInitial(); }
final class NoteListLoading extends NoteListState { const NoteListLoading(); }
final class NoteListLoaded extends NoteListState {
  const NoteListLoaded(this.notes);
  final List<Note> notes;

  NoteListLoaded withNote(Note n) => NoteListLoaded([...notes, n]);
}
final class NoteListFailure extends NoteListState {
  const NoteListFailure(this.message);
  final String message;
}
```

**Cubit** with CancelToken lifecycle:

```dart
// lib/app/modules/notes/note_list_cubit.dart
import 'package:dio/dio.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../core/errors/app_exception.dart';
import '../../data/models/note.model.dart';
import '../../data/repositories/note.repository.dart';

part 'note_list_state.dart';

class NoteListCubit extends Cubit<NoteListState> {
  NoteListCubit({required NoteRepository noteRepository})
      : _repo = noteRepository,
        super(const NoteListInitial());

  final NoteRepository _repo;
  CancelToken? _cancelToken;

  Future<void> loadNotes() async {
    if (state is NoteListLoading) return;
    emit(const NoteListLoading());
    _cancelToken?.cancel('Superseded');
    final token = _cancelToken = CancelToken();
    try {
      final notes = await _repo.listNotes(cancelToken: token);
      emit(NoteListLoaded(notes));
    } on ApiException catch (e) {
      if (token.isCancelled) return;
      emit(NoteListFailure(e.message));
    }
  }

  Future<void> createNote(String title) async {
    try {
      final note = await _repo.createNote(title);
      final current = state;
      if (current is NoteListLoaded) emit(current.withNote(note));
    } on ApiException catch (e) {
      emit(NoteListFailure(e.message));
    }
  }

  @override
  Future<void> close() {
    _cancelToken?.cancel('Cubit closed');
    return super.close();
  }
}
```

Key patterns:

- Re-entrancy guard (`if (state is NoteListLoading) return;`) on list calls.
- Cancel prior token, replace with new one.
- After `await`, check `token.isCancelled` before emitting failure — otherwise a cancelled request would emit a spurious error.
- Cancel token in `close()`.

## 5. View

```dart
// lib/app/modules/notes/notes_view.dart
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../core/utils/snackbar_helper.dart';
import 'note_list_cubit.dart';

class NotesView extends StatefulWidget {
  const NotesView({super.key});
  @override State<NotesView> createState() => _NotesViewState();
}

class _NotesViewState extends State<NotesView> {
  @override
  void initState() {
    super.initState();
    context.read<NoteListCubit>().loadNotes();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Notes')),
      body: BlocConsumer<NoteListCubit, NoteListState>(
        listener: (context, state) {
          if (state is NoteListFailure) {
            SnackbarHelper.showError(context, state.message);
          }
        },
        builder: (context, state) => switch (state) {
          NoteListLoading() => const Center(child: CircularProgressIndicator()),
          NoteListLoaded(:final notes) => ListView.builder(
              itemCount: notes.length,
              itemBuilder: (_, i) => ListTile(title: Text(notes[i].title)),
            ),
          NoteListFailure(:final message) => Center(child: Text(message)),
          _ => const SizedBox.shrink(),
        },
      ),
    );
  }
}
```

View rules:

- **Never** call a repository directly from a widget — go through the cubit.
- **`BlocListener`** handles side effects (snackbars, navigation).
- **`BlocBuilder`** handles rebuilds. Use a sealed `switch` for exhaustive state handling.
- Trigger initial load in `initState`, not in `build`.

## 6. Route

Add the path constant:

```dart
// lib/app/routes/app_routes.dart
static const notes = '/notes';
```

Add the route + cubit provider in [app_router.dart](../lib/app/core/router/app_router.dart):

```dart
GoRoute(
  path: AppRoutes.notes,
  builder: (context, _) => BlocProvider(
    create: (_) => NoteListCubit(
      noteRepository: context.read<NoteRepository>(),
    ),
    child: const NotesView(),
  ),
),
```

If it's a protected route, that's it — the existing redirect (lines 63-89) enforces sign-in. For guest-only routes, add the path to the `_authRoutes` set at the bottom of the file.

## 7. Wire into `main.dart`

Three edits in [main.dart](../lib/main.dart):

**(a)** Construct provider + repository in the dependency graph (~line 51):

```dart
final noteProvider = NoteProvider(dioService.dio);
final noteRepository = NoteRepository(noteProvider);
```

**(b)** Pass it to `MyApp`:

```dart
MyApp(
  ...
  noteRepository: noteRepository,
  ...
)
```

Add a matching `required` field on `MyApp`.

**(c)** Expose it via `RepositoryProvider`:

```dart
MultiRepositoryProvider(
  providers: [
    ...
    RepositoryProvider<NoteRepository>.value(value: widget.noteRepository),
  ],
  ...
)
```

## 8. Feature flag (optional but recommended)

Gate the home-screen entry point so consumers of the template can disable notes without deleting code. See [feature-flags.md](./feature-flags.md) for the full pattern.

```dart
// feature_flags.dart
final bool notes;
// constructor + fromEnv:
notes: _flag('FEATURE_NOTES'),
```

```env
# .env + .env.example
FEATURE_NOTES=true
```

```dart
// home_view.dart
if (flags.notes)
  ListTile(
    title: const Text('Notes'),
    onTap: () => context.push(AppRoutes.notes),
  ),
```

## 9. Test

```dart
// test/unit/note_repository_test.dart
import 'package:bloc_test/bloc_test.dart';
import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:myapp/app/data/models/note.model.dart';
import 'package:myapp/app/data/providers/note.provider.dart';
import 'package:myapp/app/data/repositories/note.repository.dart';
import 'package:myapp/app/modules/notes/note_list_cubit.dart';

class _MockProvider extends Mock implements NoteProvider {}
class _MockRepo extends Mock implements NoteRepository {}
class _FakeCancelToken extends Fake implements CancelToken {}

Response<dynamic> _response(dynamic data) => Response<dynamic>(
      requestOptions: RequestOptions(path: '/v1/api/notes'),
      statusCode: 200,
      data: data,
    );

Map<String, dynamic> _envelope(dynamic data) => {
      'success': true,
      'data': data,
      'meta': {'timestamp': '2024-01-01T00:00:00.000Z', 'requestId': 'r'},
    };

void main() {
  setUpAll(() {
    registerFallbackValue(_FakeCancelToken());
  });

  group('NoteRepository', () {
    late _MockProvider provider;
    late NoteRepository repo;

    setUp(() {
      provider = _MockProvider();
      repo = NoteRepository(provider);
    });

    test('listNotes unwraps envelope', () async {
      when(() => provider.listNotes(
            cancelToken: any(named: 'cancelToken'),
          )).thenAnswer((_) async => _response(_envelope([
                {'id': '1', 'title': 'hi', 'createdAt': '2024-01-01T00:00:00.000Z'},
              ])));

      final notes = await repo.listNotes();
      expect(notes, hasLength(1));
      expect(notes.first.id, '1');
    });
  });

  group('NoteListCubit', () {
    late _MockRepo repo;
    setUp(() => repo = _MockRepo());

    blocTest<NoteListCubit, NoteListState>(
      'loadNotes emits Loading then Loaded',
      build: () => NoteListCubit(noteRepository: repo),
      setUp: () {
        when(() => repo.listNotes(
              cancelToken: any(named: 'cancelToken'),
            )).thenAnswer((_) async => [
              Note(id: '1', title: 'hi', createdAt: DateTime(2024)),
            ]);
      },
      act: (c) => c.loadNotes(),
      expect: () => [
        isA<NoteListLoading>(),
        isA<NoteListLoaded>()
            .having((s) => s.notes.length, 'notes.length', 1),
      ],
    );
  });
}
```

Run:

```bash
flutter test test/unit/note_repository_test.dart
```

## Patterns cheat-sheet

| Situation | Pattern |
|---|---|
| Non-paginated list | `unwrapEnvelopeList` + `.map(X.fromJson).toList()` |
| Paginated list | Add `_pagination(envelope)` helper → return `({List<X> items, String? cursor, bool hasMore})` — see [admin.repository.dart](../lib/app/data/repositories/admin.repository.dart) |
| Form validation errors | Catch `ApiException`; emit `*Failure(fieldErrors: e.fieldErrors)`; consume via `TextFormField.forceErrorText` — see [sign_in_view.dart:80,98](../lib/app/modules/auth/sign_in/sign_in_view.dart#L80) |
| Guarded by role | Add a `startsWith(path)` check in the `redirect` function — see [app_router.dart:83-87](../lib/app/core/router/app_router.dart#L83-L87) |
| Needs a deep link | Add a branch in `_handleDeepLink` in [main.dart:138-175](../lib/main.dart#L138-L175) |
| Multipart upload | `FormData.fromMap({'field': await MultipartFile.fromFile(path)})` — see [user.provider.dart::uploadAvatar](../lib/app/data/providers/user.provider.dart#L18) |

## Common pitfalls

- **Forgot to register `_FakeCancelToken` fallback** → mocktail throws when the cubit passes a cancel token through `any(named: 'cancelToken')`.
- **Unwrapping auth endpoints** → auth endpoints return raw payloads, not envelopes. Don't call `unwrapEnvelope` on `/v1/api/auth/*` responses.
- **Reading flags in a widget without a `RepositoryProvider<FeatureFlags>` ancestor** → `ProviderNotFoundException`. Widget tests must wrap with `RepositoryProvider<FeatureFlags>.value(value: const FeatureFlags(...))`.
- **Emitting after cancellation** → always check `token.isCancelled` after `await` before emitting a `Failure`.
- **Storing sensitive data in `SharedPreferences`** → use [AuthService](../lib/app/services/auth_service.dart) / `flutter_secure_storage` instead.
