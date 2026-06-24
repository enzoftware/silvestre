import 'package:flutter/material.dart';
import 'package:silvestre_flutter/silvestre_flutter.dart';
import 'package:silvestre_flutter_example/src/editor/editor.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await Silvestre.init();
  runApp(const SilvestreExampleApp());
}

class SilvestreExampleApp extends StatelessWidget {
  const SilvestreExampleApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Silvestre',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorSchemeSeed: const Color(0xFF6750A4),
        useMaterial3: true,
        brightness: Brightness.light,
      ),
      darkTheme: ThemeData(
        colorSchemeSeed: const Color(0xFF6750A4),
        useMaterial3: true,
        brightness: Brightness.dark,
      ),
      home: const EditorPage(),
    );
  }
}
