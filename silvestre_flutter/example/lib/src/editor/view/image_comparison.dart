import 'dart:typed_data';

import 'package:flutter/material.dart';

class ImageComparison extends StatelessWidget {
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
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
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
