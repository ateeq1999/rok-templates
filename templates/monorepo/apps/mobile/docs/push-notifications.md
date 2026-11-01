# Firebase and push notifications (FCM)

Push notifications pick a vendor, so the App ships without them
(workspace ADRs 0023 and 0027). The recipe that adds them lives in the
templates' guide, beside the template this App comes from:
[Push notifications](https://github.com/ateeq1999/rok-data/blob/main/crates/rok-templates/docs/guide/recipes/push-notifications.md).

In short:

1. `android/app/google-services.json` and
   `ios/Runner/GoogleService-Info.plist` are placeholders for the project
   `your-firebase-project`, and nothing reads them. `flutterfire configure
   --project=<your-project-id>` replaces them with your Firebase project's
   and writes `lib/firebase_options.dart`.
2. `flutter pub add firebase_core firebase_messaging`, start Firebase in
   `main()`, and read the device's token after signing in.
3. `apps/api` has no device-token endpoint yet: add one, run `rok gen`
   from the monorepo's root, and call it through the generated client
   (`api.<tag>.<operation>(...)`), never around it.
4. In the foreground, show `FirebaseMessaging.onMessage` yourself.
