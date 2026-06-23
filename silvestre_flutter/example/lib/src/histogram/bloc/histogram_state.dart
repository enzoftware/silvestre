part of 'histogram_bloc.dart';

/// Lifecycle status of the [HistogramBloc].
enum HistogramStatus {
  /// No histogram has been requested.
  initial,

  /// A histogram computation is in flight.
  loading,

  /// A histogram is available in [HistogramState.histogram].
  success,

  /// Computation failed; [HistogramState.error] holds the message.
  failure,
}

/// State for the histogram panel.
class HistogramState extends Equatable {
  const HistogramState({
    this.status = HistogramStatus.initial,
    this.histogram,
    this.error,
  });

  /// Current lifecycle status.
  final HistogramStatus status;

  /// The most recently computed histogram, or `null` if none.
  final Histogram? histogram;

  /// Human-readable error message when [status] is [HistogramStatus.failure].
  final String? error;

  /// Whether a histogram is available to display.
  bool get isVisible => status == HistogramStatus.success && histogram != null;

  HistogramState copyWith({
    HistogramStatus? status,
    Histogram? Function()? histogram,
    String? error,
  }) {
    return HistogramState(
      status: status ?? this.status,
      histogram: histogram != null ? histogram() : this.histogram,
      error: error,
    );
  }

  @override
  List<Object?> get props => [status, histogram, error];
}
