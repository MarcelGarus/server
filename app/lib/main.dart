import 'dart:math';

import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:server/api.dart';
import 'package:syncfusion_flutter_charts/charts.dart';

import 'utils.dart';

void main() => runApp(CompanionApp());

class CompanionApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'marcelgarus.dev',
      theme: ThemeData(
        primaryColor: MyColors.pink,
        scaffoldBackgroundColor: Color(0xff111111),
        cardColor: Color(0xff222222),
      ),
      home: DashboardPage(),
    );
  }
}

class DashboardPage extends StatefulWidget {
  const DashboardPage({Key? key}) : super(key: key);

  @override
  State<DashboardPage> createState() => _DashboardPageState();
}

class _DashboardPageState extends State<DashboardPage> {
  bool _isFetching = false;
  Response? _response;

  @override
  void initState() {
    _fetch();
    super.initState();
  }

  void _fetch() async {
    if (_isFetching) return;
    setState(() => _isFetching = true);
    final r = await fetchApi();
    setState(() {
      _isFetching = false;
      _response = r;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: ListView(
        children: [
          _buildStatusCard(),
          _buildUptimeCard(),
          _buildLogSizeCard(),
          _buildVisitsPerDayCard(),
          _buildVisits(),
          SizedBox(height: 16),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: _fetch,
        label: Text('Refresh'),
        icon: Icon(Icons.refresh),
        backgroundColor: MyColors.purple,
      ),
    );
  }

  Widget _buildStatusCard() {
    return DashboardCard(
      color: MyColors.pink,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text('marcelgarus.dev', style: titleStyle),
          SizedBox(height: 8),
          Text(_status),
        ],
      ),
    );
  }

  String get _status {
    if (_isFetching) return 'Loading...';
    if (_response == null) return 'No data yet.';
    final response = _response!;
    if (response.statusCode == 200) return 'Data from ${response.timestamp}.';
    return 'Status ${response.statusCode}. Data from ${response.timestamp}.';
  }

  Widget _buildUptimeCard() {
    return DashboardCard(
      color: MyColors.green,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(_programUptime, style: titleStyle),
          Text('since the program started'),
          SizedBox(height: 8),
          Text(_response?.data?.serverUptime ?? '███'),
        ],
      ),
    );
  }

  String get _programUptime {
    final uptime = _response?.data?.programUptime;
    if (uptime == null) {
      return '██d ██h ██min ██s';
    } else {
      return '${uptime.inDays}d ${uptime.inHours % 24}h '
          '${uptime.inMinutes % 60}min '
          '${uptime.inSeconds % 60}s';
    }
  }

  Widget _buildLogSizeCard() {
    return DashboardCard(
      color: MyColors.green,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(_logFileSize, style: titleStyle),
          Text('of log data'),
        ],
      ),
    );
  }

  String get _logFileSize {
    final fileSize = _response?.data?.logFileSize;
    if (fileSize == null) return '██ B';
    if (fileSize == 0) return '0 B';
    final prefixes = ['', 'Ki', 'Mi', 'Gi', 'Ti'];
    var magnitude = (log(fileSize) / log(1024)).floor();
    if (magnitude >= prefixes.length) magnitude = prefixes.length - 1;
    return '${(fileSize / pow(1024, magnitude)).toStringAsFixed(2)} ${prefixes[magnitude]}B';
  }

  Widget _buildVisitsPerDayCard() {
    final totalNumberOfVisits = _response?.data?.visitsByDay.entries
        .map((entry) => entry.value)
        .reduce((a, b) => a + b);
    return DashboardCard(
      color: MyColors.yellow,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text('${totalNumberOfVisits ?? '██'} visits', style: titleStyle),
          Text('in the last 30 days'),
          SizedBox(height: 8),
          SizedBox(
            height: 200,
            child: SfCartesianChart(
              series: <ChartSeries>[
                StackedColumnSeries<MapEntry<DateTime, int>, String>(
                  animationDuration: 0,
                  dataSource:
                      _response?.data?.visitsByDay.entries.toList() ?? [],
                  xValueMapper: (report, _) =>
                      report.key.toIso8601String().substring(0, 10),
                  yValueMapper: (report, _) => report.value,
                  color: Colors.black,
                ),
              ],
              primaryXAxis: CategoryAxis(name: 'time'),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildVisits() {
    String subtitle(Visit visit) =>
        '${visit.timestamp.toLocal().toString().substring(0, 19)}, '
        'status ${visit.responseCode}, ${visit.handlingDuration.inMicroseconds}µs handling\n'
        '${visit.userAgent ?? '<no user agent>'}';
    return DashboardCard(
      color: MyColors.yellow,
      withPadding: false,
      child: Column(
        children: [
          SizedBox(height: 16),
          for (final visit in _response?.data?.tail ?? [null, null, null])
            ListTile(
              title: Text(
                '${visit?.method ?? '███'} ${visit?.url ?? '███████████'}',
                style: GoogleFonts.josefinSans(fontWeight: FontWeight.bold),
              ),
              subtitle: Text(visit == null ? '███████' : subtitle(visit)),
            ),
          SizedBox(height: 16),
        ],
      ),
    );
  }
}

class DashboardCard extends StatelessWidget {
  const DashboardCard({
    Key? key,
    required this.color,
    required this.child,
    this.withPadding = true,
  }) : super(key: key);

  final Widget child;
  final Color color;
  final bool withPadding;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 0),
      child: Material(
        color: color,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
        child: Padding(
          padding: withPadding ? const EdgeInsets.all(16) : EdgeInsets.zero,
          child: child,
        ),
      ),
    );
  }
}
