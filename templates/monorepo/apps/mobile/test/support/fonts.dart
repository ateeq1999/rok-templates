import 'dart:convert';

import 'package:flutter/services.dart';

/// Loads every font family the App declares (its `pubspec.yaml` `fonts:`
/// and Material's icons), so a golden test draws real glyphs, the same on
/// every machine, instead of the test font's boxes.
Future<void> loadAppFonts() async {
  final manifest =
      jsonDecode(await rootBundle.loadString('FontManifest.json'))
          as List<Object?>;
  for (final family in manifest.cast<Map<String, Object?>>()) {
    final loader = FontLoader(family['family']! as String);
    for (final font
        in (family['fonts']! as List<Object?>).cast<Map<String, Object?>>()) {
      loader.addFont(rootBundle.load(font['asset']! as String));
    }
    await loader.load();
  }
}
