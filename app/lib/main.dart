import 'package:flutter/material.dart';

void main() {
  runApp(MyApp());
}

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Server Companion App',
      theme: ThemeData(primarySwatch: Colors.blue),
      home: Scaffold(
        body: Center(
          child: Text('This is the server companion app.'),
        ),
      ),
    );
  }
}
