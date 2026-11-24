import 'dart:async';
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fonts.dart';

/// Runs before every test in `test/golden/`: loads the App's bundled fonts,
/// and compares goldens with [GlyphEdgeTolerantComparator], so a golden
/// drawn on one operating system passes on another.
Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  TestWidgetsFlutterBinding.ensureInitialized();
  await loadAppFonts();
  final local = goldenFileComparator as LocalFileComparator;
  goldenFileComparator = GlyphEdgeTolerantComparator(
    local.basedir.resolve('flutter_test_config.dart'),
  );
  await testMain();
}

/// A [LocalFileComparator] that forgives what differs between operating
/// systems with the same fonts: each platform's rasterizer shades the edges
/// of glyphs a little differently, so a few pixels change by a little.
///
/// An image still fails when one pixel changes by more than
/// [maxChannelDelta] in a channel (a glyph, an icon or a shape moved, added
/// or gone), or when more than [maxShiftedShare] of its pixels change by
/// more than [shiftDelta] (a colour changed).
class GlyphEdgeTolerantComparator extends LocalFileComparator {
  GlyphEdgeTolerantComparator(super.testFile);

  /// The most a channel of one pixel may change.
  static const maxChannelDelta = 128;

  /// A change larger than this counts toward [maxShiftedShare].
  static const shiftDelta = 32;

  /// The share of pixels that may change by more than [shiftDelta].
  static const maxShiftedShare = 0.005;

  @override
  Future<bool> compare(Uint8List imageBytes, Uri golden) async {
    final goldenBytes = Uint8List.fromList(await getGoldenBytes(golden));
    final actual = await _rgba(imageBytes);
    final expected = await _rgba(goldenBytes);
    if (actual.width == expected.width &&
        actual.height == expected.height &&
        _withinTolerance(actual.pixels, expected.pixels)) {
      return true;
    }
    // Out of tolerance: Flutter's own comparison writes the diff images
    // to `failures/` and explains the failure.
    final result = await GoldenFileComparator.compareLists(
      imageBytes,
      goldenBytes,
    );
    if (result.passed) return true;
    throw FlutterError(await generateFailureOutput(result, golden, basedir));
  }

  static bool _withinTolerance(Uint8List actual, Uint8List expected) {
    var shifted = 0;
    for (var i = 0; i < actual.length; i += 4) {
      var largest = 0;
      for (var channel = 0; channel < 4; channel++) {
        final delta = (actual[i + channel] - expected[i + channel]).abs();
        if (delta > largest) largest = delta;
      }
      if (largest > maxChannelDelta) return false;
      if (largest > shiftDelta) shifted++;
    }
    return shifted <= maxShiftedShare * (actual.length ~/ 4);
  }

  static Future<({int width, int height, Uint8List pixels})> _rgba(
    Uint8List png,
  ) async {
    final codec = await ui.instantiateImageCodec(png);
    final image = (await codec.getNextFrame()).image;
    final bytes = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
    final pixels = (
      width: image.width,
      height: image.height,
      pixels: bytes!.buffer.asUint8List(),
    );
    image.dispose();
    codec.dispose();
    return pixels;
  }
}
