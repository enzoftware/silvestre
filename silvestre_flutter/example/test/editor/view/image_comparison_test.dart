import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:silvestre_flutter_example/src/editor/view/image_comparison.dart';

// A 1x1 transparent PNG, enough for Image.memory to decode without assets.
final _pngBytes = Uint8List.fromList([
  0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, //
  0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
  0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
  0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
  0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41,
  0x54, 0x78, 0x9c, 0x62, 0x00, 0x01, 0x00, 0x00,
  0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00,
  0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae,
  0x42, 0x60, 0x82,
]);

void main() {
  Widget wrap(Widget child) =>
      MaterialApp(home: Scaffold(body: SingleChildScrollView(child: child)));

  group('ImageComparison', () {
    testWidgets('shows no mode toggle when there is no filtered result', (
      tester,
    ) async {
      await tester.pumpWidget(
        wrap(
          ImageComparison(
            originalBytes: _pngBytes,
            originalWidth: 1,
            originalHeight: 1,
          ),
        ),
      );

      expect(find.byType(SegmentedButton<ComparisonMode>), findsNothing);
      expect(find.byType(BeforeAfterSlider), findsNothing);
    });

    testWidgets('offers a slider toggle once a filtered result exists', (
      tester,
    ) async {
      await tester.pumpWidget(
        wrap(
          ImageComparison(
            originalBytes: _pngBytes,
            originalWidth: 1,
            originalHeight: 1,
            filteredBytes: _pngBytes,
            filteredWidth: 1,
            filteredHeight: 1,
          ),
        ),
      );

      expect(find.byType(SegmentedButton<ComparisonMode>), findsOneWidget);
      // Defaults to side-by-side.
      expect(find.byType(BeforeAfterSlider), findsNothing);

      await tester.tap(find.byIcon(Icons.compare));
      await tester.pumpAndSettle();

      expect(find.byType(BeforeAfterSlider), findsOneWidget);
    });

    testWidgets('hides the toggle while processing', (tester) async {
      await tester.pumpWidget(
        wrap(
          ImageComparison(
            originalBytes: _pngBytes,
            originalWidth: 1,
            originalHeight: 1,
            filteredBytes: _pngBytes,
            filteredWidth: 1,
            filteredHeight: 1,
            isProcessing: true,
          ),
        ),
      );

      expect(find.byType(SegmentedButton<ComparisonMode>), findsNothing);
    });
  });

  group('BeforeAfterSlider', () {
    testWidgets('dragging the divider moves it without throwing', (
      tester,
    ) async {
      await tester.pumpWidget(
        wrap(
          BeforeAfterSlider(
            beforeBytes: _pngBytes,
            afterBytes: _pngBytes,
            width: 4,
            height: 3,
          ),
        ),
      );

      await tester.drag(
        find.byType(BeforeAfterSlider),
        const Offset(-40, 0),
      );
      await tester.pumpAndSettle();

      expect(tester.takeException(), isNull);
    });
  });
}
