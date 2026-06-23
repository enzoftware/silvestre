part of 'editor_bloc.dart';

/// Base class for all [EditorBloc] events.
sealed class EditorEvent extends Equatable {
  const EditorEvent();

  @override
  List<Object?> get props => [];
}

/// An image was picked from the gallery and should become the new original.
final class EditorImagePicked extends EditorEvent {
  const EditorImagePicked({required this.bytes});

  /// The raw encoded bytes of the picked image (PNG, JPEG, BMP).
  final Uint8List bytes;

  @override
  List<Object?> get props => [bytes];
}

/// A filter was applied on top of the current image.
final class EditorFilterApplied extends EditorEvent {
  const EditorFilterApplied({required this.filter, required this.params});

  /// The filter definition to apply.
  final FilterDef filter;

  /// Filter-specific parameters keyed by parameter name.
  final Map<String, dynamic> params;

  @override
  List<Object?> get props => [filter, params];
}

/// The applied filters should be discarded, restoring the original image.
final class EditorReset extends EditorEvent {
  const EditorReset();
}

/// The current image should be cleared so a new one can be picked.
final class EditorCleared extends EditorEvent {
  const EditorCleared();
}

/// The current result should be exported (shared) as a PNG.
final class EditorExported extends EditorEvent {
  const EditorExported();
}
