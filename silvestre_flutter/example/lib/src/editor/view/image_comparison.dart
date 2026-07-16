import 'dart:typed_data';

import 'package:flutter/material.dart';

/// How the original and filtered images are laid out for comparison.
enum ComparisonMode {
  /// Two cards next to each other.
  sideBySide,

  /// A single image with a draggable divider revealing the filtered result.
  slider,
}

class ImageComparison extends StatefulWidget {
  const ImageComparison({
    required this.originalBytes,
    required this.originalWidth,
    required this.originalHeight,
    this.filteredBytes,
    this.filteredWidth,
    this.filteredHeight,
    this.isProcessing = false,
    super.key,
  });

  final Uint8List originalBytes;
  final int originalWidth;
  final int originalHeight;
  final Uint8List? filteredBytes;
  final int? filteredWidth;
  final int? filteredHeight;
  final bool isProcessing;

  @override
  State<ImageComparison> createState() => _ImageComparisonState();
}

class _ImageComparisonState extends State<ImageComparison> {
  ComparisonMode _mode = ComparisonMode.sideBySide;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    // The slider only makes sense once a fully-described filtered result
    // exists (bytes *and* positive dimensions) and is not being recomputed.
    // Treat bytes and dimensions as a single readiness contract so the
    // force-unwraps below are always safe.
    final hasFilteredImage =
        widget.filteredBytes != null &&
        (widget.filteredWidth ?? 0) > 0 &&
        (widget.filteredHeight ?? 0) > 0;
    final canCompare = hasFilteredImage && !widget.isProcessing;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (canCompare)
          Align(
            alignment: Alignment.centerRight,
            child: SegmentedButton<ComparisonMode>(
              showSelectedIcon: false,
              segments: const [
                ButtonSegment(
                  value: ComparisonMode.sideBySide,
                  icon: Icon(Icons.view_agenda_outlined, size: 18),
                  tooltip: 'Side by side',
                ),
                ButtonSegment(
                  value: ComparisonMode.slider,
                  icon: Icon(Icons.compare, size: 18),
                  tooltip: 'Before / after slider',
                ),
              ],
              selected: {_mode},
              onSelectionChanged: (v) => setState(() => _mode = v.first),
            ),
          ),
        if (canCompare) const SizedBox(height: 8),
        if (canCompare && _mode == ComparisonMode.slider)
          BeforeAfterSlider(
            beforeBytes: widget.originalBytes,
            afterBytes: widget.filteredBytes!,
            width: widget.filteredWidth!,
            height: widget.filteredHeight!,
          )
        else
          _SideBySide(
            theme: theme,
            originalBytes: widget.originalBytes,
            originalWidth: widget.originalWidth,
            originalHeight: widget.originalHeight,
            // Only surface the result card once bytes and dimensions agree;
            // otherwise fall back to the placeholder rather than force-unwrap.
            filteredBytes: hasFilteredImage ? widget.filteredBytes : null,
            filteredWidth: widget.filteredWidth,
            filteredHeight: widget.filteredHeight,
            isProcessing: widget.isProcessing,
          ),
      ],
    );
  }
}

class _SideBySide extends StatelessWidget {
  const _SideBySide({
    required this.theme,
    required this.originalBytes,
    required this.originalWidth,
    required this.originalHeight,
    required this.filteredBytes,
    required this.filteredWidth,
    required this.filteredHeight,
    required this.isProcessing,
  });

  final ThemeData theme;
  final Uint8List originalBytes;
  final int originalWidth;
  final int originalHeight;
  final Uint8List? filteredBytes;
  final int? filteredWidth;
  final int? filteredHeight;
  final bool isProcessing;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          child: _ImageCard(
            label: 'Original',
            imageBytes: originalBytes,
            width: originalWidth,
            height: originalHeight,
            theme: theme,
          ),
        ),
        const SizedBox(width: 12),
        Expanded(
          child:
              isProcessing
                  ? _ProcessingCard(theme: theme)
                  : filteredBytes != null
                  ? _ImageCard(
                    label: 'Result',
                    imageBytes: filteredBytes!,
                    width: filteredWidth!,
                    height: filteredHeight!,
                    theme: theme,
                  )
                  : _PlaceholderCard(theme: theme),
        ),
      ],
    );
  }
}

/// Overlays the [afterBytes] image on top of [beforeBytes] and reveals it with
/// a draggable vertical divider, producing an interactive before/after wipe.
class BeforeAfterSlider extends StatefulWidget {
  const BeforeAfterSlider({
    required this.beforeBytes,
    required this.afterBytes,
    required this.width,
    required this.height,
    super.key,
  });

  final Uint8List beforeBytes;
  final Uint8List afterBytes;
  final int width;
  final int height;

  @override
  State<BeforeAfterSlider> createState() => _BeforeAfterSliderState();
}

class _BeforeAfterSliderState extends State<BeforeAfterSlider> {
  /// Divider position as a fraction of the width, in `[0, 1]`.
  double _position = 0.5;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final aspectRatio =
        widget.height == 0 ? 1.0 : widget.width / widget.height;

    return Card(
      clipBehavior: Clip.antiAlias,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 8, 12, 4),
            child: Row(
              children: [
                Badge(
                  label: const Text('Before'),
                  backgroundColor: theme.colorScheme.secondary,
                ),
                const Spacer(),
                Badge(
                  label: const Text('After'),
                  backgroundColor: theme.colorScheme.primary,
                ),
              ],
            ),
          ),
          AspectRatio(
            aspectRatio: aspectRatio,
            child: LayoutBuilder(
              builder: (context, constraints) {
                final width = constraints.maxWidth;
                return Semantics(
                  slider: true,
                  label: 'Before / after comparison',
                  value: _positionLabel(_position),
                  increasedValue: _positionLabel((_position + 0.1).clamp(0, 1)),
                  decreasedValue: _positionLabel((_position - 0.1).clamp(0, 1)),
                  onIncrease: () => _nudge(0.1),
                  onDecrease: () => _nudge(-0.1),
                  child: GestureDetector(
                    onHorizontalDragUpdate: (details) {
                      setState(() {
                        _position = (_position + details.delta.dx / width)
                            .clamp(0.0, 1.0);
                      });
                    },
                    child: Stack(
                      fit: StackFit.expand,
                      children: [
                        // "After" (filtered) is the base layer filling the
                        // right of the divider; "Before" clips to the left.
                        Image.memory(
                          widget.afterBytes,
                          fit: BoxFit.cover,
                          gaplessPlayback: true,
                        ),
                        ClipRect(
                          clipper: _LeftClipper(_position),
                          child: Image.memory(
                            widget.beforeBytes,
                            fit: BoxFit.cover,
                            gaplessPlayback: true,
                          ),
                        ),
                        Positioned(
                          left: width * _position - 1,
                          top: 0,
                          bottom: 0,
                          child: Container(
                            width: 2,
                            color: theme.colorScheme.surface,
                          ),
                        ),
                        Positioned(
                          left: width * _position - 16,
                          top: 0,
                          bottom: 0,
                          child: Center(
                            child: Container(
                              width: 32,
                              height: 32,
                              decoration: BoxDecoration(
                                color: theme.colorScheme.surface,
                                shape: BoxShape.circle,
                                boxShadow: [
                                  BoxShadow(
                                    color: Colors.black.withValues(alpha: 0.3),
                                    blurRadius: 4,
                                  ),
                                ],
                              ),
                              child: Icon(
                                Icons.drag_indicator,
                                size: 20,
                                color: theme.colorScheme.onSurface,
                              ),
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  void _nudge(double delta) {
    setState(() => _position = (_position + delta).clamp(0.0, 1.0));
  }

  static String _positionLabel(double position) =>
      '${(position * 100).round()}% after';
}

/// Clips a child so only the region left of [fraction] of its width shows.
class _LeftClipper extends CustomClipper<Rect> {
  const _LeftClipper(this.fraction);

  final double fraction;

  @override
  Rect getClip(Size size) =>
      Rect.fromLTRB(0, 0, size.width * fraction, size.height);

  @override
  bool shouldReclip(_LeftClipper oldClipper) =>
      oldClipper.fraction != fraction;
}

class _ImageCard extends StatelessWidget {
  const _ImageCard({
    required this.label,
    required this.imageBytes,
    required this.width,
    required this.height,
    required this.theme,
  });

  final String label;
  final Uint8List imageBytes;
  final int width;
  final int height;
  final ThemeData theme;

  @override
  Widget build(BuildContext context) {
    return Card(
      clipBehavior: Clip.antiAlias,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 8, 12, 4),
            child: Row(
              children: [
                Badge(
                  label: Text(label),
                  backgroundColor:
                      label == 'Original'
                          ? theme.colorScheme.secondary
                          : theme.colorScheme.primary,
                ),
                const Spacer(),
                Text(
                  '${width}x$height',
                  style: theme.textTheme.labelSmall?.copyWith(
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                ),
              ],
            ),
          ),
          ClipRRect(
            borderRadius: const BorderRadius.vertical(
              bottom: Radius.circular(12),
            ),
            child: Image.memory(
              imageBytes,
              fit: BoxFit.contain,
              gaplessPlayback: true,
            ),
          ),
        ],
      ),
    );
  }
}

class _ProcessingCard extends StatelessWidget {
  const _ProcessingCard({required this.theme});

  final ThemeData theme;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: AspectRatio(
        aspectRatio: 1,
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const CircularProgressIndicator(),
              const SizedBox(height: 16),
              Text(
                'Processing...',
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: theme.colorScheme.onSurfaceVariant,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _PlaceholderCard extends StatelessWidget {
  const _PlaceholderCard({required this.theme});

  final ThemeData theme;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: AspectRatio(
        aspectRatio: 1,
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.image_outlined,
                size: 48,
                color: theme.colorScheme.onSurfaceVariant.withValues(
                  alpha: 0.4,
                ),
              ),
              const SizedBox(height: 8),
              Text(
                'Select a filter to see the result',
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.onSurfaceVariant,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
