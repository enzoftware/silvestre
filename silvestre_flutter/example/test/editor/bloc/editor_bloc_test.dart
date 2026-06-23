import 'dart:typed_data';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:silvestre_flutter_example/src/editor/bloc/editor_bloc.dart';

void main() {
  group('EditorBloc', () {
    final originalBytes = Uint8List.fromList([1, 2, 3]);
    final filteredBytes = Uint8List.fromList([4, 5, 6]);

    Future<void> noopShare(Uint8List bytes) async {}

    test('initial state is empty', () {
      expect(
        EditorBloc(share: noopShare).state,
        const EditorState(),
      );
    });

    group('EditorReset', () {
      blocTest<EditorBloc, EditorState>(
        'clears the filtered result, keeping the original',
        build: () => EditorBloc(share: noopShare),
        seed:
            () => EditorState(
              status: EditorStatus.ready,
              originalBytes: originalBytes,
              filteredBytes: filteredBytes,
            ),
        act: (bloc) => bloc.add(const EditorReset()),
        expect:
            () => [
              EditorState(
                status: EditorStatus.ready,
                originalBytes: originalBytes,
              ),
            ],
      );
    });

    group('EditorCleared', () {
      blocTest<EditorBloc, EditorState>(
        'resets to the empty state',
        build: () => EditorBloc(share: noopShare),
        seed:
            () => EditorState(
              status: EditorStatus.ready,
              originalBytes: originalBytes,
            ),
        act: (bloc) => bloc.add(const EditorCleared()),
        expect: () => [const EditorState()],
      );
    });

    group('EditorExported', () {
      blocTest<EditorBloc, EditorState>(
        'shares the filtered bytes when present',
        build: () {
          var shared = Uint8List(0);
          final bloc = EditorBloc(
            share: (bytes) async => shared = bytes,
          );
          addTearDown(() => expect(shared, filteredBytes));
          return bloc;
        },
        seed:
            () => EditorState(
              status: EditorStatus.ready,
              originalBytes: originalBytes,
              filteredBytes: filteredBytes,
            ),
        act: (bloc) => bloc.add(const EditorExported()),
        expect: () => <EditorState>[],
      );

      blocTest<EditorBloc, EditorState>(
        'falls back to the original bytes when no filter is applied',
        build: () {
          var shared = Uint8List(0);
          final bloc = EditorBloc(
            share: (bytes) async => shared = bytes,
          );
          addTearDown(() => expect(shared, originalBytes));
          return bloc;
        },
        seed:
            () => EditorState(
              status: EditorStatus.ready,
              originalBytes: originalBytes,
            ),
        act: (bloc) => bloc.add(const EditorExported()),
        expect: () => <EditorState>[],
      );

      blocTest<EditorBloc, EditorState>(
        'does nothing when no image is loaded',
        build: () => EditorBloc(share: (_) async => fail('should not share')),
        act: (bloc) => bloc.add(const EditorExported()),
        expect: () => <EditorState>[],
      );

      blocTest<EditorBloc, EditorState>(
        'emits failure when sharing throws',
        build:
            () => EditorBloc(
              share: (_) async => throw Exception('share failed'),
            ),
        seed:
            () => EditorState(
              status: EditorStatus.ready,
              originalBytes: originalBytes,
            ),
        act: (bloc) => bloc.add(const EditorExported()),
        expect:
            () => [
              isA<EditorState>()
                  .having((s) => s.status, 'status', EditorStatus.failure)
                  .having((s) => s.error, 'error', contains('share failed')),
            ],
      );
    });
  });
}
