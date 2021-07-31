import 'package:flutter/material.dart';
import 'package:black_hole_flutter/black_hole_flutter.dart';

import 'api.dart' as api;
import 'dev_cards.dart';

Future<void> main() async {
  await api.initialize();
  runApp(CompanionApp());
}

class CompanionApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Server Companion App',
      theme: ThemeData.dark().copyWith(
        primaryColor: Color(0xfff97191),
        primaryColorLight: Color(0xfff97191).hsv.withValue(0.9).toColor(),
        primaryColorDark: Color(0xfff97191).hsv.withValue(0.2).toColor(),
        accentColor: Color(0xfff97191),
        scaffoldBackgroundColor: Color(0xff111111),
        cardColor: Color(0xff222222),
      ),
      home: LayoutBuilder(builder: (context, constraints) {
        if (constraints.maxWidth < 1000) {
          return DefaultTabController(
            length: 3,
            child: Scaffold(
              bottomNavigationBar: TabBar(
                tabs: [
                  Tab(icon: Icon(Icons.storage_outlined), text: 'Dev'),
                  Tab(icon: Icon(Icons.article_outlined), text: 'Content'),
                  Tab(icon: Icon(Icons.bolt_outlined), text: 'Shortcuts'),
                ],
              ),
              body: TabBarView(
                children: [
                  TabPage(cards: devCards),
                  TabPage(cards: []),
                  TabPage(cards: []),
                ],
              ),
            ),
          );
        } else {
          return Scaffold(
            body: ListView(
              children: [
                Row(
                  children: [
                    SizedBox(width: 8),
                    Expanded(child: DashboardColumn(cards: devCards)),
                    SizedBox(width: 8),
                    Expanded(child: Container()),
                    SizedBox(width: 8),
                    Expanded(child: Container()),
                    SizedBox(width: 8),
                  ],
                ),
              ],
            ),
          );
        }
      }),
    );
  }
}

class TabPage extends StatelessWidget {
  const TabPage({Key? key, required this.cards}) : super(key: key);

  final List<Widget> cards;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: EdgeInsets.fromLTRB(8, 8, 8, 0),
      children: [
        for (final card in cards) ...[
          card,
          SizedBox(height: 8),
        ],
      ],
    );
  }
}

class DashboardColumn extends StatelessWidget {
  const DashboardColumn({Key? key, required this.cards}) : super(key: key);

  final List<Widget> cards;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        SizedBox(height: 8),
        for (final card in cards) ...[
          card,
          SizedBox(height: 8),
        ],
      ],
    );
  }
}
