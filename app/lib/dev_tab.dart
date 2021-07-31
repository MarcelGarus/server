import 'dart:math';

import 'package:flutter/material.dart';
import 'package:syncfusion_flutter_charts/charts.dart';
import 'package:black_hole_flutter/black_hole_flutter.dart';
import 'package:dartx/dartx.dart';
import 'package:tuple/tuple.dart';

import 'api.dart' as api;

class DevTab extends StatelessWidget {
  const DevTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ListView(
      children: [
        Padding(
          padding: EdgeInsets.all(16),
          child: UserAgentGraph(),
        ),
        FutureBuilder<List<api.Visit>>(
          future: api.visitsTail(),
          builder: (context, snapshot) {
            if (!snapshot.hasData && !snapshot.hasError) {
              return Center(child: CircularProgressIndicator());
            } else if (snapshot.hasError) {
              return Center(child: Text(snapshot.error.toString()));
            } else {
              return Column(
                children: [
                  for (final visit in snapshot.requireData) VisitTile(visit),
                ],
              );
            }
          },
        ),
      ],
    );
  }
}

class UserAgentGraph extends StatelessWidget {
  const UserAgentGraph({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Material(
      elevation: 12,
      borderRadius: BorderRadius.circular(16),
      child: Padding(
        padding: EdgeInsets.all(8),
        child: FutureBuilder<Map<DateTime, Map<String, int>>>(
          future: api.visitsUserAgents(),
          builder: (context, snapshot) {
            if (!snapshot.hasData && !snapshot.hasError) {
              return Container();
            }
            if (snapshot.hasError) {
              return Text(snapshot.error.toString());
            }
            return _buildDiagram(context, snapshot.requireData);
          },
        ),
      ),
    );
  }

  Widget _buildDiagram(
      BuildContext context, Map<DateTime, Map<String, int>> data) {
    // Relatively consistent requests should be shown at the bottom.
    // For example, StatusCake tests or Googlebot crawls should be at
    // the bottom of the chart so that more varying user agents are on
    // the top and easier to see. That's why we calculate how much each
    // user agent distribution deviates from its mean.
    final userAgents = data.values.expand((map) => map.keys).toSet();
    final averages = <String, double>{
      for (final userAgent in userAgents)
        userAgent: data.entries
                .map((entry) => entry.value[userAgent] ?? 0)
                .reduce((a, b) => a + b) /
            data.length,
    };
    final deviations = <String, double>{
      for (final userAgent in userAgents)
        userAgent: () {
          final average = averages[userAgent]!;
          return data.entries
              .map((entry) => ((entry.value[userAgent] ?? 0) - average).abs())
              .reduce((a, b) => a + b);
        }(),
    };
    final userAgentsSortedByConsistency = userAgents.toList()
      ..sort((a, b) {
        final difference = deviations[a]! - deviations[b]!;
        if (difference < 0) return -1;
        if (difference > 0) return 1;
        return 0;
      });

    final botColors = [
      for (var i = 0.0; i < 1.0; i += 0.1)
        Color.lerp(Colors.purple, Color(0xfff97191), i)!
    ];
    final otherColors = [Colors.amber, Colors.orange];
    final userAgentNamesAndColors = <String, Tuple2<String?, Color>>{
      for (final userAgent in userAgents)
        userAgent: () {
          if (userAgent.contains('StatusCake')) {
            return Tuple2<String?, Color>('StatusCake', Color(0xff5af1c8));
          }
          if (userAgent.contains('PostmanRuntime')) {
            return Tuple2<String?, Color>('Postman', Color(0xffff6c37));
          }
          if (userAgent == 'CompanionApp') {
            return Tuple2<String?, Color>(
                '(this companion app)', Color(0xff999999));
          }
          if (userAgent.toLowerCase().contains('bot')) {
            return Tuple2<String?, Color>(
              null,
              botColors.isEmpty ? Colors.pink : botColors.removeAt(0),
            );
          }

          final random = Random(userAgent.hashCode);
          return Tuple2<String?, Color>(
            null,
            otherColors.isNotEmpty
                ? otherColors.removeAt(0)
                : Color.fromARGB(255, random.nextInt(255), random.nextInt(255),
                    random.nextInt(255)),
          );
        }(),
    };

    return Column(
      children: [
        SizedBox(
          height: 300,
          child: SfCartesianChart(
            series: <ChartSeries>[
              for (final userAgent in userAgentsSortedByConsistency)
                StackedColumnSeries<MapEntry<DateTime, Map<String, int>>, int>(
                  dataSource: data.entries.toList(),
                  xValueMapper: (report, _) =>
                      report.key.millisecondsSinceEpoch,
                  yValueMapper: (report, _) => report.value[userAgent],
                  color: userAgentNamesAndColors[userAgent]!.item2,
                ),
            ],
            primaryXAxis: CategoryAxis(name: 'time'),
          ),
        ),
        for (final userAgent in userAgents) ...[
          SizedBox(height: 8),
          Row(
            children: [
              Container(
                width: 16,
                height: 16,
                decoration: BoxDecoration(
                  color: userAgentNamesAndColors[userAgent]!.item2,
                  borderRadius: BorderRadius.circular(8),
                ),
              ),
              SizedBox(width: 16),
              Expanded(
                child: Text(
                  userAgentNamesAndColors[userAgent]!.item1 ?? userAgent,
                ),
              ),
            ],
          ),
        ],
      ],
    );
  }
}

class VisitTile extends StatelessWidget {
  const VisitTile(this.visit, {Key? key}) : super(key: key);

  final api.Visit visit;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      leading: CircleAvatar(child: Text(visit.responseCode.toString())),
      title: Text('${visit.method} ${visit.url}'),
      subtitle: Text(visit.userAgent ?? '<no user agent>'),
      trailing: Text(
        '${visit.handlingDuration.inMicroseconds} µs\n'
        '${visit.timestamp.toLocal().toString().substring(0, 19)}',
        textAlign: TextAlign.right,
      ),
    );
  }
}
