import 'dart:async';
import 'dart:convert';

import 'package:api_client/api_client.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:mobile/app/data/repositories/live.repository.dart';

import '../support/api.dart';

/// An API whose `GET /api/v1/live` streams answer from `streams`, one per
/// connection, recording each ticket asked for in `tickets`.
LiveRepository liveApi(
  List<Stream<String>> streams, {
  required List<String> tickets,
  required List<Uri> opened,
}) {
  var connection = 0;
  return LiveRepository(
    url: (path) => Uri.parse('$apiOrigin$path'),
    session: Session(token: () => 'tok-1'),
    httpClient: () => MockClient.streaming((request, _) async {
      if (request.method == 'POST') {
        tickets.add(request.headers['Authorization'] ?? '');
        final body = jsonEncode({
          'ticket': 't-${tickets.length}',
          'expires_in': 60,
        });
        return http.StreamedResponse(Stream.value(utf8.encode(body)), 200);
      }
      opened.add(request.url);
      final events = streams[connection++].map(utf8.encode);
      return http.StreamedResponse(events, 200);
    }),
    firstRetry: Duration.zero,
  );
}

String event(String topic, Object? payload) =>
    'data: ${jsonEncode({'topic': topic, 'payload': payload})}\n\n';

void main() {
  test('notices streams the events of the subscribed Topics', () async {
    final tickets = <String>[];
    final opened = <Uri>[];
    final live = StreamController<String>();
    final api = liveApi([live.stream], tickets: tickets, opened: opened);

    final notices = api.notices(['orders.new', 'bills.paid']);
    final first = notices.first;
    live
      ..add(': keep-alive\n\n')
      ..add('data: {"topic":"orders.new",')
      ..add('"payload":{"id":1}}\n\n');
    final notice = await first;

    expect(notice.topic, 'orders.new');
    expect(notice.payload, {'id': 1});
    expect(tickets, ['Bearer tok-1']);
    expect(opened.single.path, '/api/v1/live');
    expect(opened.single.queryParameters, {
      'topics': 'orders.new,bills.paid',
      'ticket': 't-1',
    });
    await live.close();
  });

  test('a dropped connection is reopened with a fresh ticket', () async {
    final tickets = <String>[];
    final opened = <Uri>[];
    final second = StreamController<String>();
    final api = liveApi(
      [Stream.value(event('orders.new', 1)), second.stream],
      tickets: tickets,
      opened: opened,
    );

    final received = <Notice>[];
    final done = Completer<void>();
    final subscription = api.notices(['orders.new']).listen((notice) {
      received.add(notice);
      if (received.length == 2) done.complete();
    });
    await Future<void>.delayed(const Duration(milliseconds: 50));
    second.add(event('orders.new', 2));
    await done.future.timeout(const Duration(seconds: 5));

    expect(received.map((notice) => notice.payload), [1, 2]);
    expect(opened.map((uri) => uri.queryParameters['ticket']), ['t-1', 't-2']);
    await subscription.cancel();
    await second.close();
  });
}
