import 'package:flutter/material.dart';
import 'package:silvestre_flutter/silvestre_flutter.dart';

class HistogramView extends StatelessWidget {
  const HistogramView({required this.histogram, super.key});

  final Histogram histogram;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final channelColors = _channelColors(histogram.numChannels);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Histogram', style: theme.textTheme.titleSmall),
            const SizedBox(height: 8),
            SizedBox(
              height: 120,
              child: CustomPaint(
                size: const Size(double.infinity, 120),
                painter: _HistogramPainter(
                  histogram: histogram,
                  channelColors: channelColors,
                ),
              ),
            ),
            const SizedBox(height: 12),
            ...List.generate(histogram.numChannels, (c) {
              final stats = histogram.stats[c];
              return Padding(
                padding: const EdgeInsets.symmetric(vertical: 2),
                child: Row(
                  children: [
                    Container(
                      width: 10,
                      height: 10,
                      decoration: BoxDecoration(
                        color: channelColors[c],
                        shape: BoxShape.circle,
                      ),
                    ),
                    const SizedBox(width: 8),
                    Text(
                      _channelName(c, histogram.numChannels),
                      style: theme.textTheme.labelSmall,
                    ),
                    const Spacer(),
                    Text(
                      'min=${stats.min}  max=${stats.max}  '
                      '\u03BC=${stats.mean.toStringAsFixed(1)}  '
                      '\u03C3=${stats.stdDev.toStringAsFixed(1)}',
                      style: theme.textTheme.labelSmall?.copyWith(
                        fontFamily: 'monospace',
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                  ],
                ),
              );
            }),
          ],
        ),
      ),
    );
  }

  List<Color> _channelColors(int channels) {
    switch (channels) {
      case 1:
        return [Colors.grey];
      case 3:
        return [Colors.red, Colors.green, Colors.blue];
      case 4:
        return [Colors.red, Colors.green, Colors.blue, Colors.grey];
      default:
        return List.generate(channels, (_) => Colors.grey);
    }
  }

  String _channelName(int index, int total) {
    if (total == 1) return 'Luma';
    const names = ['Red', 'Green', 'Blue', 'Alpha'];
    return index < names.length ? names[index] : 'Ch$index';
  }
}

class _HistogramPainter extends CustomPainter {
  _HistogramPainter({required this.histogram, required this.channelColors});

  final Histogram histogram;
  final List<Color> channelColors;

  @override
  void paint(Canvas canvas, Size size) {
    if (histogram.bins.isEmpty) return;

    // Find global max bin value across all channels for normalization.
    var maxVal = 0;
    for (final ch in histogram.bins) {
      for (final v in ch) {
        if (v > maxVal) maxVal = v;
      }
    }
    if (maxVal == 0) return;

    final binWidth = size.width / 256;

    for (var c = 0; c < histogram.numChannels && c < 4; c++) {
      final paint =
          Paint()
            ..color = channelColors[c].withValues(alpha: 0.5)
            ..style = PaintingStyle.fill;

      final path = Path()..moveTo(0, size.height);
      for (var i = 0; i < 256; i++) {
        final h = (histogram.bins[c][i] / maxVal) * size.height;
        path.lineTo(i * binWidth, size.height - h);
      }
      path
        ..lineTo(size.width, size.height)
        ..close();
      canvas.drawPath(path, paint);
    }
  }

  @override
  bool shouldRepaint(covariant _HistogramPainter old) =>
      old.histogram != histogram;
}
