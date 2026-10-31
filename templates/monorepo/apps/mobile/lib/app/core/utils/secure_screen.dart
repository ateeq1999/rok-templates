import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:flutter_windowmanager/flutter_windowmanager.dart';

/// Mix into a [State] to block screenshots / screen recording while the
/// screen is visible.
///
/// Android: sets `FLAG_SECURE` via [FlutterWindowManager]. The flag is
/// cleared in [dispose], so the rest of the app is unaffected.
///
/// iOS: OS-level screenshot prevention requires an app-wide blur overlay
/// on `AppLifecycleState.inactive`. Left as a project-level decision —
/// see [docs/security.md] for the recommended approach.
mixin SecureScreenMixin<T extends StatefulWidget> on State<T> {
  @override
  void initState() {
    super.initState();
    if (Platform.isAndroid) {
      FlutterWindowManager.addFlags(FlutterWindowManager.FLAG_SECURE);
    }
  }

  @override
  void dispose() {
    if (Platform.isAndroid) {
      FlutterWindowManager.clearFlags(FlutterWindowManager.FLAG_SECURE);
    }
    super.dispose();
  }
}
