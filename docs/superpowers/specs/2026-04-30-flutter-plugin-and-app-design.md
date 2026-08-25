# Flutter Plugin & Mobile App Design Specification

**Date:** 2026-04-30  
**Topic:** Flutter Plugin Architecture, flutter_rust_bridge v2, and BLoC Mobile App  
**Pull Requests:** #61, #63  
**Status:** Implemented  

---

## 1. Overview

This specification details `silvestre_flutter`, a cross-platform Flutter plugin that binds `silvestre-core` to Dart using `flutter_rust_bridge` (FRB) v2, alongside an example mobile application using the BLoC architecture pattern.

---

## 2. Plugin Architecture (`silvestre_flutter/`)

### 2.1 Native Bridging via `flutter_rust_bridge` v2
- Rust API defined in `silvestre_flutter/rust/src/api/image.rs`.
- `flutter_rust_bridge_codegen` generates asynchronous Dart bindings with zero main-thread blocking.
- Platform channels:
  - **Android:** Compiles native `.so` binaries via `cargo-ndk`.
  - **iOS / macOS:** Compiles static C archive `.a` and links via CocoaPods podspec.

### 2.2 Dart API Layer (`lib/`)

```dart
abstract class Silvestre {
  static Future<SilvestreImage> fromBytes(Uint8List bytes, int width, int height);
  static Future<SilvestreImage> loadFromFile(String path);
}

abstract class SilvestreImage {
  Future<SilvestreImage> applyGaussian({double sigma = 1.5});
  Future<SilvestreImage> applyCanny({double low = 50.0, double high = 150.0});
  Future<SilvestreImage> applyGrayscale();
  Future<Uint8List> toJpeg({int quality = 85});
  Future<void> save(String path);
}
```

---

## 3. Example Mobile Application (`silvestre_flutter/example/`)

### 3.1 State Management (BLoC Pattern)
- `ImageBloc`: Manages image loading, filter pipeline processing in background isolates, and UI state (Initial, Loading, Processed, Error).
- Prevents UI frame drops and micro-stutters during heavy image convolutions.

### 3.2 Interactive UI Features
1. **Camera Feed & Photo Picker:** Direct image capture from device camera or selection from photo library.
2. **Interactive Before/After Slider:** Custom split-view widget allowing horizontal dragging to compare the original image against the filtered result.
3. **Device Gallery Export:** High-resolution image export and save to native camera roll.

---

## 4. Verification

- `flutter test` running widget and unit tests.
- Platform integration tests verifying Rust bridge communications on Android emulator and iOS simulator.
