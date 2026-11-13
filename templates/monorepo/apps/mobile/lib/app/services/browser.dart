import 'package:url_launcher/url_launcher.dart';

/// Opens a web page outside the App: a provider's sign-in page, which ends
/// by opening the App's deep link. Tests record where it was sent instead.
abstract interface class Browser {
  /// Opens `url` in the system browser.
  Future<void> open(Uri url);
}

/// The phone's own browser, so the provider's page is the real one (and
/// its saved sign-in), never a web view the App could read.
final class SystemBrowser implements Browser {
  const SystemBrowser();

  @override
  Future<void> open(Uri url) async {
    final opened = await launchUrl(url, mode: LaunchMode.externalApplication);
    if (!opened) throw StateError('No browser could open $url');
  }
}
