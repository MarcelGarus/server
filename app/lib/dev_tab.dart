import 'dart:math';

import 'package:flutter/material.dart';
import 'package:syncfusion_flutter_charts/charts.dart';
import 'package:black_hole_flutter/black_hole_flutter.dart';
import 'package:dartx/dartx.dart';

import 'api.dart' as api;
import 'utils.dart';

class DevTab extends StatelessWidget {
  const DevTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: EdgeInsets.all(8),
      children: [
        UserAgentGraph(),
        SizedBox(height: 8),
        VisitsTrail(),
      ],
    );
  }
}

class UserAgentGraph extends StatelessWidget {
  const UserAgentGraph({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return DashboardCard(
      title: 'Server load',
      builder: (context, isDetailed) {
        return FutureBuilder<Map<DateTime, Map<String, int>>>(
          future: api.visitsUserAgents(),
          builder: (context, snapshot) {
            if (!snapshot.hasData && !snapshot.hasError) {
              return Container();
            }
            if (snapshot.hasError) {
              return Text(snapshot.error.toString());
            }
            return _buildDiagram(context, isDetailed, snapshot.requireData);
          },
        );
      },
    );
  }

  Widget _buildDiagram(
    BuildContext context,
    bool isDetailed,
    Map<DateTime, Map<String, int>> data,
  ) {
    final userAgentInfos = data.values
        .expand((map) => map.keys)
        .toSet()
        .map((userAgent) => UserAgentInfo.from(userAgent))
        .sortedBy((userAgent) => userAgent.type.index);

    final botColors = [Colors.purple, Color(0xfff97191)].cycle().iterator;
    final humanColors = [Colors.amber, Colors.orange].cycle().iterator;
    Color colorFor(UserAgentInfo info) {
      switch (info.type) {
        case UserAgentType.uptimeMonitoring:
          return Color(0xff5af1c8);
        case UserAgentType.bot:
          return botColors.next();
        case UserAgentType.debugging:
          if (info.userAgent.toLowerCase().contains('postmanruntime'))
            return Color(0xffff6c37);
          if (info.userAgent.toLowerCase().contains('companionapp'))
            return Color(0xff444444);
          return Colors.green; // Unknown debugging user agent.
        case UserAgentType.human:
          return humanColors.next();
      }
    }

    final colors = {
      for (final info in userAgentInfos) info.userAgent: colorFor(info)
    };

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          height: 300,
          child: SfCartesianChart(
            series: <ChartSeries>[
              for (final info in userAgentInfos)
                StackedColumnSeries<MapEntry<DateTime, Map<String, int>>, int>(
                  animationDuration: 0,
                  dataSource: data.entries.toList(),
                  xValueMapper: (report, _) =>
                      report.key.millisecondsSinceEpoch,
                  yValueMapper: (report, _) => report.value[info.userAgent],
                  color: colors[info.userAgent]!,
                ),
            ],
            primaryXAxis: CategoryAxis(name: 'time'),
          ),
        ),
        if (isDetailed)
          Padding(
            padding: EdgeInsets.fromLTRB(8, 0, 8, 8),
            child: Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                for (final info in userAgentInfos.reversed)
                  ActionChip(
                    backgroundColor: colors[info.userAgent]!,
                    label: Text(
                      info.simpleName ?? info.userAgent,
                      style: TextStyle(
                          color: colors[info.userAgent]!.highEmphasisOnColor),
                    ),
                    onPressed: () => context.showSimpleSnackBar(info.userAgent),
                  ),
              ],
            ),
          ),
      ],
    );
  }
}

class VisitsTrail extends StatelessWidget {
  const VisitsTrail({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return DashboardCard(
      title: 'Last visits',
      builder: (context, isDetailed) {
        return FutureBuilder<List<api.Visit>>(
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
        );
      },
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
