import 'dart:convert';
import 'dart:typed_data';

import 'src/rust/api/analysis.dart' as rust_analysis;
import 'src/rust/api/filters.dart' as rust_filters;
import 'src/rust/api/image.dart' as rust_image;

export 'src/rust/frb_generated.dart' show RustLib;

/// Image color space.
enum ColorSpace {
  /// 4 channels: red, green, blue, alpha.
  rgba,

  /// 3 channels: red, green, blue.
  rgb,

  /// 1 channel: luminance.
  grayscale;

  String _toRustString() {
    switch (this) {
      case ColorSpace.rgba:
        return 'rgba';
      case ColorSpace.rgb:
        return 'rgb';
      case ColorSpace.grayscale:
        return 'grayscale';
    }
  }

  static ColorSpace _fromRustString(String s) {
    switch (s) {
      case 'rgba':
        return ColorSpace.rgba;
      case 'rgb':
        return ColorSpace.rgb;
      case 'grayscale':
        return ColorSpace.grayscale;
      default:
        throw ArgumentError('Unknown color space: $s');
    }
  }
}

/// Image encoding format.
enum ImageFormat {
  /// PNG format (lossless).
  png,

  /// JPEG format (lossy).
  jpeg,

  /// BMP format (uncompressed).
  bmp;

  String _toRustString() {
    switch (this) {
      case ImageFormat.png:
        return 'png';
      case ImageFormat.jpeg:
        return 'jpeg';
      case ImageFormat.bmp:
        return 'bmp';
    }
  }
}

/// A silvestre image backed by a Rust-side opaque handle.
///
/// All operations are async and execute on a background isolate via
/// flutter_rust_bridge. Filter operations return new [SilvestreImage]
/// instances without mutating the original.
class SilvestreImage {
  final rust_image.SilvestreImageWrapper _inner;
  final int _width;
  final int _height;
  final ColorSpace _colorSpace;

  SilvestreImage._(this._inner, this._width, this._height, this._colorSpace);

  /// Load an image from a file path (PNG, JPEG, BMP).
  static Future<SilvestreImage> fromPath(String path) async {
    final wrapper = await rust_image.loadImageFromPath(path: path);
    return _wrap(wrapper);
  }

  /// Load an image from raw encoded bytes (PNG, JPEG, BMP).
  static Future<SilvestreImage> fromBytes(Uint8List bytes) async {
    final wrapper = await rust_image.loadImageFromBytes(bytes: bytes);
    return _wrap(wrapper);
  }

  /// Create an image from raw pixel data.
  static Future<SilvestreImage> create(
    Uint8List pixels,
    int width,
    int height,
    ColorSpace colorSpace,
  ) async {
    final wrapper = await rust_image.createImage(
      pixels: pixels,
      width: width,
      height: height,
      colorSpace: colorSpace._toRustString(),
    );
    return SilvestreImage._(wrapper, width, height, colorSpace);
  }

  /// Image width in pixels.
  int get width => _width;

  /// Image height in pixels.
  int get height => _height;

  /// Image color space.
  ColorSpace get colorSpace => _colorSpace;

  /// Get a copy of the raw pixel data.
  Future<Uint8List> get pixels => rust_image.imagePixels(img: _inner);

  // ---------- Filters (generic) ------------------------------------------

  /// Apply a named filter, returning a new image.
  ///
  /// See the filter table in the README for available filter names and
  /// their parameters.
  Future<SilvestreImage> applyFilter(
    String name, {
    Map<String, dynamic>? params,
  }) async {
    final paramsJson = params != null ? jsonEncode(params) : '';
    final wrapper = await rust_filters.applyFilter(
      img: _inner,
      name: name,
      paramsJson: paramsJson,
    );
    return _wrap(wrapper);
  }

  // ---------- Convenience filter methods ---------------------------------

  /// Convert to grayscale.
  Future<SilvestreImage> grayscale() => applyFilter('grayscale');

  /// Invert colors.
  Future<SilvestreImage> invert() => applyFilter('invert');

  /// Apply sepia tone.
  Future<SilvestreImage> sepia() => applyFilter('sepia');

  /// Adjust brightness by [delta] (-255 to 255).
  Future<SilvestreImage> brightness(int delta) =>
      applyFilter('brightness', params: {'delta': delta});

  /// Adjust contrast by [factor] (0.0 = gray, 1.0 = unchanged, >1.0 = more).
  Future<SilvestreImage> contrast(double factor) =>
      applyFilter('contrast', params: {'factor': factor});

  /// Apply sharpening filter.
  Future<SilvestreImage> sharpen() => applyFilter('sharpen');

  /// Apply box blur.
  Future<SilvestreImage> boxBlur() => applyFilter('box_blur');

  /// Apply Sobel edge detection.
  Future<SilvestreImage> sobel() => applyFilter('sobel');

  /// Apply Gaussian blur with the given [sigma].
  Future<SilvestreImage> gaussian(double sigma) =>
      applyFilter('gaussian', params: {'sigma': sigma});

  /// Apply median filter with the given kernel [size].
  Future<SilvestreImage> median(int size) =>
      applyFilter('median', params: {'size': size});

  /// Apply Canny edge detection.
  Future<SilvestreImage> canny({
    required double low,
    required double high,
    required double sigma,
  }) =>
      applyFilter('canny', params: {'low': low, 'high': high, 'sigma': sigma});

  /// Crop to the given rectangle.
  Future<SilvestreImage> crop(int x, int y, int w, int h) =>
      applyFilter('crop', params: {'x': x, 'y': y, 'w': w, 'h': h});

  /// Resize to the given dimensions using bilinear interpolation.
  Future<SilvestreImage> resize(int w, int h) =>
      applyFilter('resize', params: {'w': w, 'h': h});

  /// Rotate by [angle] degrees.
  Future<SilvestreImage> rotate(double angle) =>
      applyFilter('rotate', params: {'angle': angle});

  /// Mirror the image.
  ///
  /// [mode] must be one of `"horizontal"`, `"vertical"`, or `"both"`.
  Future<SilvestreImage> mirror(String mode) =>
      applyFilter('mirror', params: {'mode': mode});

  // ---------- I/O --------------------------------------------------------

  /// Save the image to a file path.
  Future<void> save(String path, {ImageFormat format = ImageFormat.png}) =>
      rust_image.saveImage(
        img: _inner,
        path: path,
        format: format._toRustString(),
      );

  /// Encode the image to in-memory bytes.
  Future<Uint8List> encode({ImageFormat format = ImageFormat.png}) =>
      rust_image.encodeImage(img: _inner, format: format._toRustString());

  // ---------- Analysis ---------------------------------------------------

  /// Compute a per-channel histogram.
  Future<Histogram> computeHistogram() async {
    final result = await rust_analysis.computeHistogram(img: _inner);
    return Histogram._fromRust(result);
  }

  /// Compute a single-channel luminance histogram (BT.601).
  ///
  /// Returns `null` for unsupported color spaces.
  Future<Histogram?> computeLuminanceHistogram() async {
    final result = await rust_analysis.computeLuminanceHistogram(img: _inner);
    if (result == null) return null;
    return Histogram._fromRust(result);
  }

  // ---------- Internal ---------------------------------------------------

  static Future<SilvestreImage> _wrap(
    rust_image.SilvestreImageWrapper wrapper,
  ) async {
    final w = await rust_image.imageWidth(img: wrapper);
    final h = await rust_image.imageHeight(img: wrapper);
    final csStr = await rust_image.imageColorSpace(img: wrapper);
    final cs = ColorSpace._fromRustString(csStr);
    return SilvestreImage._(wrapper, w, h, cs);
  }

  /// Provides access to the underlying FRB wrapper for advanced use.
  rust_image.SilvestreImageWrapper get inner => _inner;
}

/// Per-channel statistics from a histogram computation.
class ChannelStats {
  /// Minimum intensity value in the channel.
  final int min;

  /// Maximum intensity value in the channel.
  final int max;

  /// Arithmetic mean of pixel intensities.
  final double mean;

  /// Population standard deviation.
  final double stdDev;

  const ChannelStats({
    required this.min,
    required this.max,
    required this.mean,
    required this.stdDev,
  });
}

/// Per-channel histogram and statistics for an image.
class Histogram {
  /// Intensity bins per channel. `bins[channel]` has 256 entries.
  final List<List<int>> bins;

  /// Per-channel statistics.
  final List<ChannelStats> stats;

  /// Total number of pixels counted.
  final int pixelCount;

  const Histogram._({
    required this.bins,
    required this.stats,
    required this.pixelCount,
  });

  /// Number of channels.
  int get numChannels => bins.length;

  factory Histogram._fromRust(rust_analysis.HistogramResult result) {
    return Histogram._(
      bins: result.bins.map((b) => b.map((v) => v.toInt()).toList()).toList(),
      stats:
          result.stats
              .map(
                (s) => ChannelStats(
                  min: s.min,
                  max: s.max,
                  mean: s.mean,
                  stdDev: s.stdDev,
                ),
              )
              .toList(),
      pixelCount: result.pixelCount.toInt(),
    );
  }
}
