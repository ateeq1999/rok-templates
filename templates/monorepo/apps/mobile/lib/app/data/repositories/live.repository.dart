import 'dart:async';
import 'dart:convert';

import 'package:api_client/api_client.dart';
import 'package:http/http.dart' as http;

/// One piece of news on a Topic, such as `orders.new`: a screen showing
/// what it concerns reloads it.
final class Notice {
  const Notice(this.topic, this.payload);

  /// The Topic, such as `orders.new`.
  final String topic;

  /// What changed, such as `{"id": "…"}`; `null` for none.
  final Object? payload;
}

/// Live updates over Topics: the API pushes a notice when something
/// changes, so a screen reloads on news instead of polling. The stream is
/// `GET /api/v1/live` (server-sent events), opened with a short-lived
/// ticket from `POST /api/v1/live/tickets`; neither is in the generated
/// client, as an event stream has no schema to generate from.
class LiveRepository {
  /// Live updates from the API whose paths `url` resolves (the client's
  /// [ApiClient.url]), asking tickets with `session`'s token.
  /// `httpClient` makes the HTTP client of each connection, closed when
  /// the connection ends.
  LiveRepository({
    required Uri Function(String path) url,
    required Session session,
    http.Client Function()? httpClient,
    this.firstRetry = const Duration(seconds: 1),
    this.lastRetry = const Duration(seconds: 30),
  }) : _url = url,
       _session = session,
       _httpClient = httpClient ?? http.Client.new;

  /// The wait before reopening a dropped connection; it doubles with each
  /// failure in a row, up to [lastRetry].
  final Duration firstRetry;

  /// The longest wait before reopening a dropped connection.
  final Duration lastRetry;

  final Uri Function(String path) _url;
  final Session _session;
  final http.Client Function() _httpClient;

  /// The notices on `topics`, as they come, until the subscription is
  /// cancelled. A dropped connection is reopened with a fresh ticket, so
  /// the stream never ends by itself; reload what is shown when it
  /// reopens if a missed notice matters.
  Stream<Notice> notices(List<String> topics) {
    late final StreamController<Notice> controller;
    http.Client? client;
    var cancelled = false;

    Future<void> run() async {
      var wait = firstRetry;
      while (!cancelled) {
        final connection = client = _httpClient();
        try {
          final ticket = await _ticket(connection);
          final query = {'topics': topics.join(','), 'ticket': ticket};
          final request = http.Request(
            'GET',
            _url('/api/v1/live').replace(queryParameters: query),
          )..headers['Accept'] = 'text/event-stream';
          final response = await connection.send(request);
          if (response.statusCode != 200) {
            throw http.ClientException('live: ${response.statusCode}');
          }
          wait = firstRetry;
          await for (final data in _events(response.stream)) {
            final notice = _notice(data);
            if (notice != null && !cancelled) controller.add(notice);
          }
        } on Object {
          // Dropped, refused or offline: reopen after the wait.
        } finally {
          connection.close();
        }
        if (cancelled) return;
        await Future<void>.delayed(wait);
        final doubled = wait * 2;
        wait = doubled > lastRetry ? lastRetry : doubled;
      }
    }

    controller = StreamController<Notice>(
      onListen: () => unawaited(run()),
      onCancel: () {
        cancelled = true;
        client?.close();
      },
    );
    return controller.stream;
  }

  /// A ticket to the caller's Topics.
  Future<String> _ticket(http.Client client) async {
    final token = await _session.token();
    final response = await client.post(
      _url('/api/v1/live/tickets'),
      headers: {if (token != null) 'Authorization': 'Bearer $token'},
    );
    if (response.statusCode != 200) {
      throw http.ClientException('live: no ticket (${response.statusCode})');
    }
    final body = jsonDecode(response.body) as Map<String, Object?>;
    return body['ticket']! as String;
  }

  /// The `data` of each event in `bytes`, a server-sent event stream.
  static Stream<String> _events(Stream<List<int>> bytes) async* {
    final data = <String>[];
    await for (final line
        in bytes.transform(utf8.decoder).transform(const LineSplitter())) {
      if (line.isEmpty) {
        if (data.isNotEmpty) yield data.join('\n');
        data.clear();
      } else if (line.startsWith('data:')) {
        data.add(line.substring(5).trimLeft());
      }
    }
  }

  static Notice? _notice(String data) {
    try {
      final json = jsonDecode(data) as Map<String, Object?>;
      return Notice(json['topic']! as String, json['payload']);
    } on Object {
      return null;
    }
  }
}
