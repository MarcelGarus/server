import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

abstract class MyColors {
  MyColors._();

  static const pink = Color(0xfff97191);
  static const yellow = Color(0xffffb152);
  static const green = Color(0xff7ece64);
  static const purple = Color(0xffc59bff);
}

final titleStyle = GoogleFonts.josefinSans(
  fontSize: 28,
  fontWeight: FontWeight.bold,
);
