import 'dart:typed_data';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:silvestre_flutter_example/src/editor/bloc/editor_bloc.dart';

void main() {
  group('EditorBloc', () {
    final originalBytes = Uint8List.fromList([1, 2, 3]);
    final filteredBytes = Uint8List.fromList([4, 5, 6]);

    Future<void> noopShare(Uint8List bytes) async {}
    Future<void> noopSave(Uint8List bytes) async {}

    test('initial state is empty', () {
      expect(
        EditorBloc(share: noopShare, save: noopSave).state,
        const EditorState(),
      );
    });

    group('EditorReset', () {
      blocTest<EditorBloc, EditorState>(
        'clears the filtered result, keeping the original',
        build: () => EditorBloc(share: noopShare, save: noopSave),
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
        build: () => EditorBloc(share: noopShare, save: noopSave),
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
            save: noopSave,
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
            save: noopSave,
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
        build:
            () => EditorBloc(
              share: (_) async => fail('should not share'),
              save: noopSave,
            ),
        act: (bloc) => bloc.add(const EditorExported()),
        expect: () => <EditorState>[],
      );

      blocTest<EditorBloc, EditorState>(
        'emits failure when sharing throws',
        build:
            () => EditorBloc(
              share: (_) async => throw Exception('share failed'),
              save: noopSave,
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

    group('EditorSaved', () {
      blocTest<EditorBloc, EditorState>(
        'saves the filtered bytes and emits saved then ready',
        build: () {
          var saved = Uint8List(0);
          final bloc = EditorBloc(
            share: noopShare,
            save: (bytes) async => saved = bytes,
          );
          addTearDown(() => expect(saved, filteredBytes));
          return bloc;
        },
        seed:
            () => EditorState(
              status: EditorStatus.ready,
              originalBytes: originalBytes,
              filteredBytes: filteredBytes,
            ),
        act: (bloc) => bloc.add(const EditorSaved()),
        expect:
            () => [
              isA<EditorState>().having(
                (s) => s.status,
                'status',
                EditorStatus.processing,
              ),
              isA<EditorState>()
                  .having((s) => s.status, 'status', EditorStatus.saved)
                  .having(
                    (s) => s.originalBytes,
                    'originalBytes',
                    originalBytes,
                  )
                  .having(
                    (s) => s.filteredBytes,
                    'filteredBytes',
                    filteredBytes,
                  ),
              isA<EditorState>()
                  .having((s) => s.status, 'status', EditorStatus.ready)
                  .having(
                    (s) => s.originalBytes,
                    'originalBytes',
                    originalBytes,
                  )
                  .having(
                    (s) => s.filteredBytes,
                    'filteredBytes',
                    filteredBytes,
                  ),
            ],
      );

      blocTest<EditorBloc, EditorState>(
        'falls back to the original bytes when no filter is applied',
        build: () {
          var saved = Uint8List(0);
          final bloc = EditorBloc(
            share: noopShare,
            save: (bytes) async => saved = bytes,
          );
          addTearDown(() => expect(saved, originalBytes));
          return bloc;
        },
        seed:
            () => EditorState(
              status: EditorStatus.ready,
              originalBytes: originalBytes,
            ),
        act: (bloc) => bloc.add(const EditorSaved()),
        expect:
            () => [
              isA<EditorState>().having(
                (s) => s.status,
                'status',
                EditorStatus.processing,
              ),
              isA<EditorState>()
                  .having((s) => s.status, 'status', EditorStatus.saved)
                  .having(
                    (s) => s.originalBytes,
                    'originalBytes',
                    originalBytes,
                  ),
              isA<EditorState>()
                  .having((s) => s.status, 'status', EditorStatus.ready)
                  .having(
                    (s) => s.originalBytes,
                    'originalBytes',
                    originalBytes,
                  ),
            ],
      );

      blocTest<EditorBloc, EditorState>(
        'ignores a save request while another operation is in flight',
        build:
            () => EditorBloc(
              share: noopShare,
              save: (_) async => fail('should not save while processing'),
            ),
        seed:
            () => EditorState(
              status: EditorStatus.processing,
              originalBytes: originalBytes,
            ),
        act: (bloc) => bloc.add(const EditorSaved()),
        expect: () => <EditorState>[],
      );

      blocTest<EditorBloc, EditorState>(
        'does nothing when no image is loaded',
        build:
            () => EditorBloc(
              share: noopShare,
              save: (_) async => fail('should not save'),
            ),
        act: (bloc) => bloc.add(const EditorSaved()),
        expect: () => <EditorState>[],
      );

      blocTest<EditorBloc, EditorState>(
        'emits failure when saving throws',
        build:
            () => EditorBloc(
              share: noopShare,
              save: (_) async => throw Exception('save failed'),
            ),
        seed:
            () => EditorState(
              status: EditorStatus.ready,
              originalBytes: originalBytes,
            ),
        act: (bloc) => bloc.add(const EditorSaved()),
        expect:
            () => [
              isA<EditorState>().having(
                (s) => s.status,
                'status',
                EditorStatus.processing,
              ),
              isA<EditorState>()
                  .having((s) => s.status, 'status', EditorStatus.failure)
                  .having((s) => s.error, 'error', contains('save failed')),
            ],
      );
    });
  });
}
