part of 'editor_bloc.dart';

/// Lifecycle status of the [EditorBloc].
enum EditorStatus {
  /// No image has been loaded yet.
  empty,

  /// An image is loaded and ready for editing.
  ready,

  /// A filter or export operation is in flight.
  processing,

  /// The last operation failed; [EditorState.error] holds the message.
  failure,
}

/// Immutable state for the image editor.
///
/// A single-class state is used because every field is relevant across the
/// whole editing flow (original, current result, and processing status all
/// coexist on screen at once).
class EditorState extends Equatable {
  const EditorState({
    this.status = EditorStatus.empty,
    this.original,
    this.originalBytes,
    this.filtered,
    this.filteredBytes,
    this.error,
  });

  /// Current lifecycle status.
  final EditorStatus status;

  /// The unmodified image as loaded by the user.
  final SilvestreImage? original;

  /// PNG-encoded bytes of [original], ready for display.
  final Uint8List? originalBytes;

  /// The most recent filtered result, or `null` if no filter is applied.
  final SilvestreImage? filtered;

  /// PNG-encoded bytes of [filtered], ready for display.
  final Uint8List? filteredBytes;

  /// Human-readable error message when [status] is [EditorStatus.failure].
  final String? error;

  /// The image a new filter should be applied to: the filtered result if one
  /// exists, otherwise the original. `null` when no image is loaded.
  SilvestreImage? get current => filtered ?? original;

  /// PNG bytes that should be exported: the filtered result if present,
  /// otherwise the original.
  Uint8List? get currentBytes => filteredBytes ?? originalBytes;

  /// Whether an image is currently loaded.
  bool get hasImage => original != null;

  /// Whether a filter has been applied on top of the original.
  bool get hasResult => filtered != null;

  EditorState copyWith({
    EditorStatus? status,
    SilvestreImage? original,
    Uint8List? originalBytes,
    SilvestreImage? Function()? filtered,
    Uint8List? Function()? filteredBytes,
    String? error,
  }) {
    return EditorState(
      status: status ?? this.status,
      original: original ?? this.original,
      originalBytes: originalBytes ?? this.originalBytes,
      filtered: filtered != null ? filtered() : this.filtered,
      filteredBytes:
          filteredBytes != null ? filteredBytes() : this.filteredBytes,
      error: error,
    );
  }

  @override
  List<Object?> get props => [
    status,
    original,
    originalBytes,
    filtered,
    filteredBytes,
    error,
  ];
}
