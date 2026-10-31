# Recipe: Firebase and push notifications (FCM)

Push notifications pick a vendor, so the App ships without them and this
recipe adds them (workspace ADRs 0023 and 0027): Firebase Cloud Messaging
on the App's side, and a device-token endpoint on `apps/api`'s, which it
does not serve yet.

## What ships

- `android/app/google-services.json` and
  `ios/Runner/GoogleService-Info.plist`: placeholders for the project
  `your-firebase-project`. Nothing reads them: the Android build does not
  apply the Google services plugin, and the App does not start Firebase.
- No Firebase package, no `lib/firebase_options.dart`, no `firebase.json`.

Firebase configuration names one Firebase project (its API keys and app
ids), which is why a template cannot ship a real one. Every build carries
it, so it is not a secret; restrict the API keys to your App in the Google
Cloud console.

## 1. Make the configuration real

With a Firebase project of your own:

```bash
dart pub global activate flutterfire_cli
flutterfire configure --project=<your-project-id>
```

It registers the App's Android `applicationId` and iOS bundle id with the
project, replaces the two placeholders, and writes `lib/firebase_options.dart`
and `firebase.json`. It also adds the Google services plugin to
`android/settings.gradle.kts` and `android/app/build.gradle.kts`.

## 2. Start Firebase and ask for a token

```bash
flutter pub add firebase_core firebase_messaging
```

<!-- not compiled: a recipe; adapt it to lib/main.dart -->
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

iOS also needs the Push Notifications capability and background mode
"Remote notifications" in Xcode, and an APNs key uploaded to the Firebase
project.

## 3. Give the API the token

`apps/api` has no device-token endpoint yet. Add one as a feature of its
own (a `device_tokens` table: user, token, platform; `POST` to register,
`DELETE` on sign-out), send with the FCM HTTP v1 API from a job, and run
`rok gen` from the monorepo's root: the client gains the operation, and
the App calls it through `api.<tag>.<operation>(...)` like every other
call. Never call the API around the generated client.

## 4. Show a message in the foreground

FCM shows notifications by itself only while the App is in the background.
In the foreground, listen to `FirebaseMessaging.onMessage` and show it
(a snackbar, or `flutter_local_notifications` for a system notification).
