import 'package:flutter/material.dart';
import 'package:black_hole_flutter/black_hole_flutter.dart';

import 'api.dart' as api;
import 'dev_tab.dart';

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
      home: DefaultTabController(
        length: 3,
        child: Scaffold(
          bottomNavigationBar: TabBar(
            tabs: [
              Tab(
                icon: Icon(Icons.storage_outlined),
                text: 'Dev',
              ),
              Tab(
                icon: Icon(Icons.bolt_outlined),
                text: 'Shortcuts',
              ),
              Tab(
                icon: Icon(Icons.article_outlined),
                text: 'Content',
              ),
            ],
          ),
          body: TabBarView(
            children: [
              DevTab(),
              EmptyTab(),
              EmptyTab(),
            ],
          ),
        ),
      ),
    );
  }
}

class EmptyTab extends StatelessWidget {
  const EmptyTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container();
  }
}
