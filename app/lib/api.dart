import 'dart:convert';

import 'package:http/http.dart' as http;

import 'admin_key.dart';

const _adminApi = 'https://marcelgarus.dev/admin';

Future<Response> fetchApi() async {
  final response = await http.get(
    Uri.parse(_adminApi),
    headers: {
      'Admin-Key': adminKey,
      'User-Agent': 'CompanionApp',
    },
  );
  AdminData? data;
  if (response.statusCode == 200) {
    try {
      data = AdminData.fromJson(json.decode(response.body));
    } catch (error) {
      print('Parsing the data failed: $error');
      print('Original data: ${response.body}');
    }
  }
  return Response(response.statusCode, data);
}

class Response {
  final int statusCode;
  final DateTime timestamp;
  final AdminData? data;

  Response(this.statusCode, this.data) : timestamp = DateTime.now();
}

class AdminData {
  final String serverUptime;
  final Duration programUptime;
  final int logFileSize;
  final List<Visit> tail;
  final Map<DateTime, int> visitsByDay;

  AdminData({
    required this.serverUptime,
    required this.programUptime,
    required this.logFileSize,
    required this.tail,
    required this.visitsByDay,
  });

  AdminData.fromJson(dynamic json)
      : this(
          serverUptime: json['server_uptime'] as String,
          programUptime: Duration(
            seconds: json['server_program_uptime'] as int,
          ),
          logFileSize: json['log_file_size'] as int,
          tail: (json['visits_tail'] as List<dynamic>)
              .map(Visit.fromJson)
              .toList(),
          visitsByDay: {
            for (final entry
                in (json['number_of_visits_by_day'] as Map<String, dynamic>)
                    .entries)
              DateTime.parse(entry.key): entry.value as int,
          },
        );
}

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
