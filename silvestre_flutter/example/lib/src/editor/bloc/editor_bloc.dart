import 'dart:typed_data';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:silvestre_flutter/silvestre_flutter.dart';
import 'package:silvestre_flutter_example/src/filters.dart';

part 'editor_event.dart';
part 'editor_state.dart';

/// Signature for a function that shares encoded PNG [bytes] with the OS.
typedef ImageSharer = Future<void> Function(Uint8List bytes);

/// Manages the image editing lifecycle: loading an image, applying filters on
/// top of one another, resetting, and exporting the result.
///
/// All image work runs in `silvestre_flutter`, which executes on a background
/// isolate, so the bloc simply awaits those futures without blocking the UI.
class EditorBloc extends Bloc<EditorEvent, EditorState> {
  EditorBloc({required ImageSharer share})
    : _share = share,
      super(const EditorState()) {
    on<EditorImagePicked>(_onImagePicked);
    on<EditorFilterApplied>(_onFilterApplied);
    on<EditorReset>(_onReset);
    on<EditorCleared>(_onCleared);
    on<EditorExported>(_onExported);
  }

  final ImageSharer _share;

  Future<void> _onImagePicked(
    EditorImagePicked event,
    Emitter<EditorState> emit,
  ) async {
    emit(state.copyWith(status: EditorStatus.processing));
    try {
      final image = await SilvestreImage.fromBytes(event.bytes);
      final bytes = await image.encode();
      emit(
        EditorState(
          status: EditorStatus.ready,
          original: image,
          originalBytes: bytes,
        ),
      );
    } on Exception catch (e) {
      emit(state.copyWith(status: EditorStatus.failure, error: '$e'));
    }
  }

  Future<void> _onFilterApplied(
    EditorFilterApplied event,
    Emitter<EditorState> emit,
  ) async {
    final source = state.current;
    if (source == null) return;

    emit(state.copyWith(status: EditorStatus.processing));
    try {
      final result = await source.applyFilter(
        event.filter.name,
        params: event.params.isEmpty ? null : event.params,
      );
      final bytes = await result.encode();
      emit(
        state.copyWith(
          status: EditorStatus.ready,
          filtered: () => result,
          filteredBytes: () => bytes,
        ),
      );
    } on Exception catch (e) {
      emit(state.copyWith(status: EditorStatus.failure, error: '$e'));
    }
  }

  void _onReset(EditorReset event, Emitter<EditorState> emit) {
    emit(
      state.copyWith(
        status: EditorStatus.ready,
        filtered: () => null,
        filteredBytes: () => null,
      ),
    );
  }

  void _onCleared(EditorCleared event, Emitter<EditorState> emit) {
    emit(const EditorState());
  }

  Future<void> _onExported(
    EditorExported event,
    Emitter<EditorState> emit,
  ) async {
    final bytes = state.currentBytes;
    if (bytes == null) return;

    try {
      await _share(bytes);
    } on Exception catch (e) {
      emit(state.copyWith(status: EditorStatus.failure, error: '$e'));
    }
  }
}
