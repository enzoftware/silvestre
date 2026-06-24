import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:silvestre_flutter_example/src/histogram/histogram.dart';

void main() {
  group('HistogramBloc', () {
    test('initial state is initial with no histogram', () {
      final bloc = HistogramBloc();
      expect(bloc.state.status, HistogramStatus.initial);
      expect(bloc.state.histogram, isNull);
      expect(bloc.state.isVisible, isFalse);
    });

    group('HistogramHidden', () {
      blocTest<HistogramBloc, HistogramState>(
        'resets to the initial state',
        build: HistogramBloc.new,
        seed: () => const HistogramState(status: HistogramStatus.loading),
        act: (bloc) => bloc.add(const HistogramHidden()),
        expect: () => [const HistogramState()],
      );
    });
  });
}
