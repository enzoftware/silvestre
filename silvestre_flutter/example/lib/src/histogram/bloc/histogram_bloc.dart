import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:silvestre_flutter/silvestre_flutter.dart';

part 'histogram_event.dart';
part 'histogram_state.dart';

/// Computes per-channel histograms for whichever image is currently on screen.
///
/// Kept separate from the editor so histogram analysis (a Flutter-only extra
/// beyond the web demo) can be toggled and recomputed without coupling to the
/// filter pipeline. The target image is passed in via the event, so there is
/// no direct bloc-to-bloc dependency.
class HistogramBloc extends Bloc<HistogramEvent, HistogramState> {
  HistogramBloc() : super(const HistogramState()) {
    on<HistogramRequested>(_onRequested);
    on<HistogramHidden>(_onHidden);
  }

  Future<void> _onRequested(
    HistogramRequested event,
    Emitter<HistogramState> emit,
  ) async {
    emit(state.copyWith(status: HistogramStatus.loading));
    try {
      final histogram = await event.image.computeHistogram();
      emit(
        state.copyWith(
          status: HistogramStatus.success,
          histogram: () => histogram,
        ),
      );
    } on Exception catch (e) {
      emit(state.copyWith(status: HistogramStatus.failure, error: '$e'));
    }
  }

  void _onHidden(HistogramHidden event, Emitter<HistogramState> emit) {
    emit(const HistogramState());
  }
}
