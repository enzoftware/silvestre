import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:image_picker/image_picker.dart';
import 'package:path_provider/path_provider.dart';
import 'package:share_plus/share_plus.dart';
import 'package:silvestre_flutter_example/src/editor/bloc/editor_bloc.dart';
import 'package:silvestre_flutter_example/src/editor/view/filter_panel.dart';
import 'package:silvestre_flutter_example/src/editor/view/image_comparison.dart';
import 'package:silvestre_flutter_example/src/histogram/histogram.dart';

/// Entry point for the editor feature.
///
/// Provides the [EditorBloc] and [HistogramBloc] to the subtree and wires up
/// the platform image sharer. The actual UI lives in [EditorView].
class EditorPage extends StatelessWidget {
  const EditorPage({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: [
        BlocProvider(create: (_) => EditorBloc(share: _sharePng)),
        BlocProvider(create: (_) => HistogramBloc()),
      ],
      child: const EditorView(),
    );
  }

  /// Writes [bytes] to a temporary PNG and opens the native share sheet.
  static Future<void> _sharePng(Uint8List bytes) async {
    final dir = await getTemporaryDirectory();
    final file = File(
      '${dir.path}/silvestre-${DateTime.now().millisecondsSinceEpoch}.png',
    );
    await file.writeAsBytes(bytes);
    await Share.shareXFiles([
      XFile(file.path, mimeType: 'image/png', name: 'silvestre-output.png'),
    ]);
  }
}

/// The editor UI: picks an image, applies filters, shows a histogram, and
/// exports the result. All state is read from [EditorBloc]/[HistogramBloc].
class EditorView extends StatelessWidget {
  const EditorView({super.key});

  Future<void> _pickImage(BuildContext context) async {
    final file = await ImagePicker().pickImage(source: ImageSource.gallery);
    if (file == null || !context.mounted) return;
    final bytes = await file.readAsBytes();
    if (!context.mounted) return;
    context.read<EditorBloc>().add(EditorImagePicked(bytes: bytes));
  }

  @override
  Widget build(BuildContext context) {
    return BlocListener<EditorBloc, EditorState>(
      listenWhen:
          (previous, current) =>
              current.status == EditorStatus.failure &&
              previous.status != EditorStatus.failure,
      listener: (context, state) {
        ScaffoldMessenger.of(context)
          ..hideCurrentSnackBar()
          ..showSnackBar(
            SnackBar(
              content: Text('Error: ${state.error}'),
              behavior: SnackBarBehavior.floating,
            ),
          );
      },
      child: Scaffold(
        appBar: AppBar(
          title: const Text('silvestre'),
          centerTitle: true,
          actions: [
            BlocBuilder<EditorBloc, EditorState>(
              builder: (context, state) {
                if (!state.hasImage) return const SizedBox.shrink();
                return Row(
                  children: [
                    IconButton(
                      icon: const Icon(Icons.bar_chart),
                      tooltip: 'Histogram',
                      onPressed: () {
                        final image = state.current;
                        if (image == null) return;
                        context.read<HistogramBloc>().add(
                          HistogramRequested(image: image),
                        );
                      },
                    ),
                    if (state.hasResult)
                      IconButton(
                        icon: const Icon(Icons.refresh),
                        tooltip: 'Reset filters',
                        onPressed: () {
                          context.read<EditorBloc>().add(const EditorReset());
                          context.read<HistogramBloc>().add(
                            const HistogramHidden(),
                          );
                        },
                      ),
                    IconButton(
                      icon: const Icon(Icons.ios_share),
                      tooltip: 'Export PNG',
                      onPressed:
                          () => context.read<EditorBloc>().add(
                            const EditorExported(),
                          ),
                    ),
                  ],
                );
              },
            ),
          ],
        ),
        body: BlocBuilder<EditorBloc, EditorState>(
          builder: (context, state) {
            if (!state.hasImage) {
              return _UploadPanel(onPick: () => _pickImage(context));
            }
            return _Editor(state: state, onPick: () => _pickImage(context));
          },
        ),
      ),
    );
  }
}

class _UploadPanel extends StatelessWidget {
  const _UploadPanel({required this.onPick});

  final VoidCallback onPick;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Card(
          child: InkWell(
            onTap: onPick,
            borderRadius: BorderRadius.circular(12),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 48, vertical: 64),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    Icons.add_photo_alternate_outlined,
                    size: 64,
                    color: theme.colorScheme.primary,
                  ),
                  const SizedBox(height: 16),
                  Text(
                    'Choose an image to get started',
                    style: theme.textTheme.titleMedium,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Supports PNG, JPEG, and BMP',
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
                  const SizedBox(height: 24),
                  FilledButton.icon(
                    onPressed: onPick,
                    icon: const Icon(Icons.folder_open),
                    label: const Text('Browse files'),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _Editor extends StatelessWidget {
  const _Editor({required this.state, required this.onPick});

  final EditorState state;
  final VoidCallback onPick;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final isWide = constraints.maxWidth > 720;
        if (isWide) {
          return Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(flex: 3, child: _ImageArea(state: state)),
              SizedBox(
                width: 320,
                child: _Sidebar(state: state, onPick: onPick),
              ),
            ],
          );
        }
        return SingleChildScrollView(
          child: Column(
            children: [
              _ImageArea(state: state),
              _Sidebar(state: state, onPick: onPick),
            ],
          ),
        );
      },
    );
  }
}

class _ImageArea extends StatelessWidget {
  const _ImageArea({required this.state});

  final EditorState state;

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(12),
      child: Column(
        children: [
          ImageComparison(
            originalBytes: state.originalBytes!,
            originalWidth: state.original!.width,
            originalHeight: state.original!.height,
            filteredBytes: state.filteredBytes,
            filteredWidth: state.filtered?.width,
            filteredHeight: state.filtered?.height,
            isProcessing: state.status == EditorStatus.processing,
          ),
          BlocBuilder<HistogramBloc, HistogramState>(
            builder: (context, hist) {
              if (!hist.isVisible) return const SizedBox.shrink();
              return Padding(
                padding: const EdgeInsets.only(top: 12),
                child: HistogramView(histogram: hist.histogram!),
              );
            },
          ),
        ],
      ),
    );
  }
}

class _Sidebar extends StatelessWidget {
  const _Sidebar({required this.state, required this.onPick});

  final EditorState state;
  final VoidCallback onPick;

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.fromLTRB(0, 12, 12, 12),
      child: Card(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            FilterPanel(
              isProcessing: state.status == EditorStatus.processing,
              onApply:
                  (filter, params) => context.read<EditorBloc>().add(
                    EditorFilterApplied(filter: filter, params: params),
                  ),
            ),
            const Divider(),
            Padding(
              padding: const EdgeInsets.all(16),
              child: OutlinedButton.icon(
                onPressed: () {
                  context.read<EditorBloc>().add(const EditorCleared());
                  context.read<HistogramBloc>().add(const HistogramHidden());
                  onPick();
                },
                icon: const Icon(Icons.image),
                label: const Text('Load different image'),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
