# Changelog

All notable changes to the `silvestre_flutter` package will be documented in this file.

## 0.1.0

### Added
- Initial release of `silvestre_flutter`.
- Pure Rust image processing engine integration via `flutter_rust_bridge` v2.
- Support for 15 image processing algorithms:
  - **Spatial Filters**: Box Blur, Gaussian Blur, Median, Sharpen, Sobel, Canny Edge Detection.
  - **Color Effects**: Grayscale, Sepia, Invert, Brightness, Contrast.
  - **Transforms**: Resize (Nearest-Neighbor & Bilinear), Rotate (Fixed & Arbitrary angles), Mirror (Horizontal, Vertical, Both), Crop.
  - **Analysis**: Per-channel & luminance histograms with statistical calculations.
- Asynchronous background isolate execution keeping Flutter UI smooth (60/120 FPS).
- Multi-platform native FFI support for Android and iOS.
- Example Flutter mobile application featuring real-time camera capture, photo gallery export, and interactive before/after split view comparison.
