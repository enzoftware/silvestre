part of 'histogram_bloc.dart';

/// Base class for all [HistogramBloc] events.
sealed class HistogramEvent extends Equatable {
  const HistogramEvent();

  @override
  List<Object?> get props => [];
}

/// A histogram should be computed for the given [image].
final class HistogramRequested extends HistogramEvent {
  const HistogramRequested({required this.image});

  /// The image to analyze.
  final SilvestreImage image;

  @override
  List<Object?> get props => [image];
}

/// The histogram panel should be hidden and its state cleared.
final class HistogramHidden extends HistogramEvent {
  const HistogramHidden();
}
