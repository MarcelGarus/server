import 'dart:convert';
import 'dart:io';

import 'package:toml/toml.dart';
import 'package:http/http.dart' as http;

const _baseApiUrl = 'https://marcelgarus.dev/api';
late final String _adminKey;

final _cache = <String, String>{};

Future<void> initialize() async {
  _adminKey = await _loadAdminKey();
}

Future<String> _loadAdminKey() async {
  final content = await File('../Config.toml').readAsString();
  final doc = TomlDocument.parse(content).toMap();
  return doc['admin_key'];
}

Future<dynamic> _fetchJson(String apiPath) async {
  final url = '$_baseApiUrl$apiPath';
  if (_cache.containsKey(url)) {
    return json.decode(_cache[url]!);
  }
  print('Fetching $url');
  final response = await http.get(
    Uri.parse(url),
    headers: {
      'Admin-Key': _adminKey,
      'User-Agent': 'CompanionApp',
    },
  );
  if (response.statusCode != 200) {
    throw 'Got a ${response.statusCode} response: ${response.body}';
  }
  _cache[url] = response.body;
  return json.decode(response.body);
}

// The Visits API

class Visit {
  final DateTime timestamp;
  final Duration handlingDuration;
  final int? responseCode;
  final String? responseError;
  final String method;
  final String url;
  final String? userAgent;
  final String? language;
  final String? referer;

  Visit({
    required this.timestamp,
    required this.handlingDuration,
    required this.responseCode,
    required this.responseError,
    required this.method,
    required this.url,
    required this.userAgent,
    required this.language,
    required this.referer,
  });

  Visit.fromJson(dynamic json)
      : this(
          timestamp: DateTime.fromMillisecondsSinceEpoch(
              (json['timestamp'] as int) * 1000),
          handlingDuration:
              Duration(microseconds: json['handlingDuration'] as int),
          responseCode:
              (json['responseStatus'] as Map<String, dynamic>)['Ok'] as int?,
          responseError: (json['responseStatus'] as Map<String, dynamic>)['Err']
              as String?,
          method: json['method'] as String,
          url: json['url'] as String,
          userAgent: json['userAgent'] as String?,
          language: json['language'] as String?,
          referer: json['referer'] as String?,
        );
}

Future<List<Visit>> visitsTail() async {
  final json = await _fetchJson('/visits/tail');
  return (json as List<dynamic>).map((json) => Visit.fromJson(json)).toList();
}

Future<Map<DateTime, Map<String, int>>> visitsUserAgents() async {
  final json = await _fetchJson('/visits/user-agents');
  return (json as Map<String, dynamic>).map((date, userAgents) {
    return MapEntry(
      DateTime.parse(date),
      userAgents.cast<String, int>(),
    );
  }).cast<DateTime, Map<String, int>>();
}
