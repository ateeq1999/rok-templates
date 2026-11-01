# Recipe: push notifications (Firebase Cloud Messaging)

Push notifications pick a vendor, so the [monorepo
template](../monorepo.md)'s mobile App ships without them and this recipe
adds them (workspace ADRs 0023 and 0027): Firebase Cloud Messaging (FCM)
in `apps/mobile`, and a device-token endpoint in `apps/api`, which it does
not serve yet. The App reaches that endpoint through the generated Dart
client, like every other call.

## What ships

- `apps/mobile/android/app/google-services.json` and
  `apps/mobile/ios/Runner/GoogleService-Info.plist`: placeholders for the
  project `your-firebase-project`. Nothing reads them: the Android build
  does not apply the Google services plugin, and the App does not start
  Firebase.
- No Firebase package, no `lib/firebase_options.dart`, no `firebase.json`.

Firebase configuration names one Firebase project (its API keys and app
ids), which is why a template cannot ship a real one. Every build carries
it, so it is not a secret; restrict the API keys to your App in the Google
Cloud console.

## 1. Make the configuration real

`rok init --org` already named the App: its Android `applicationId` and
iOS bundle id are what Firebase registers. With a Firebase project of your
own, from `apps/mobile`:

```console
$ dart pub global activate flutterfire_cli
$ flutterfire configure --project=<your-project-id>
```

It registers the App's Android `applicationId` and iOS bundle id with the
project, replaces the two placeholders, and writes `lib/firebase_options.dart`
and `firebase.json`. It also adds the Google services plugin to
`android/settings.gradle.kts` and `android/app/build.gradle.kts`.

## 2. Start Firebase and ask for a token

```console
$ flutter pub add firebase_core firebase_messaging
```

<!-- not compiled: a recipe; adapt it to apps/mobile/lib/main.dart -->
```dart
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_messaging/firebase_messaging.dart';

import 'firebase_options.dart';

@pragma('vm:entry-point')
Future<void> _onBackgroundMessage(RemoteMessage message) async {
  await Firebase.initializeApp(options: DefaultFirebaseOptions.currentPlatform);
}

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await Firebase.initializeApp(options: DefaultFirebaseOptions.currentPlatform);
  FirebaseMessaging.onBackgroundMessage(_onBackgroundMessage);
  // ... the rest of main()
}
```

After signing in, ask for permission (iOS, Android 13+) and read the
token:

<!-- not compiled: a recipe -->
```dart
final messaging = FirebaseMessaging.instance;
final settings = await messaging.requestPermission();
if (settings.authorizationStatus == AuthorizationStatus.authorized) {
  final token = await messaging.getToken();
  // send it to the API (step 3); listen to messaging.onTokenRefresh too
}
```

iOS also needs the Push Notifications capability and the background mode
"Remote notifications" in Xcode, and an APNs key uploaded to the Firebase
project.

## 3. Give the API the token

`apps/api` has no device-token endpoint yet. Add one as a feature of its
own:

- a `device_tokens` table in `apps/api/schema/` (the user, the token, the
  platform), staged with `rok gen --migration add_device_tokens` from the
  monorepo's root;
- a `POST` that registers the signed-in user's token and a `DELETE` that
  forgets it on sign-out, documented in the API's OpenAPI document like
  its other routes;
- the sending itself as a job ([Jobs and schedules](../jobs.md)) that
  calls the FCM HTTP v1 API with a service account, so a slow or failing
  send never holds up a request.

The same `rok gen` re-exports `apps/api/openapi.json` and regenerates the
clients: `clients/dart` gains the operations, and the App calls them as
`api.<tag>.<operation>(...)` (the web and desktop Apps get them too,
through `clients/ts` and `clients/rust`). Never call the API around the
generated client.

## 4. Show a message in the foreground

FCM shows notifications by itself only while the App is in the background.
In the foreground, listen to `FirebaseMessaging.onMessage` and show it
(a snackbar, or `flutter_local_notifications` for a system notification).
