# Silvestre Architecture Overview

Silvestre is a cross-platform image processing library implemented in pure Rust, designed with a modular architecture that exposes identical core algorithms across CLI, WebAssembly, Flutter (iOS & Android), and native C/C++ environments.

---

## 1. System Topology

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Platform Targets                               │
│                                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐ │
│  │silvestre-cli │  │silvestre_    │  │silvestre-wasm│  │  silvestre-ffi  │ │
│  │              │  │flutter       │  │              │  │                 │ │
│  │ (Interactive │  │ (Flutter BLoC│  │(wasm-bindgen │  │ (C ABI,         │ │
│  │   Ratatui    │  │  + FRB v2)   │  │  + Web Demo) │  │  cbindgen)      │ │
│  │     TUI)     │  │              │  │              │  │                 │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘ │
│         │                 │                 │                   │           │
│         │                 │                 │                   │ (C ABI)   │
│         │                 │                 │                   ▼           │
│         │                 │                 │          ┌─────────────────┐  │
│         │                 │                 │          │ Native Consumers│  │
│         │                 │                 │          │ (C, C++, Swift, │  │
│         │                 │                 │          │  Android NDK)   │  │
│         │                 │                 │          └─────────────────┘  │
│         │                 │                 │                               │
│         └────────────┬────┴─────────────────┴───────────────────────────────┘
│                      ▼
│  ┌──────────────────────────────────────────────────────────────────────────┐
│  │                              silvestre-core                              │
│  │                         (Pure Rust, No Platform Deps)                    │
│  │                                                                          │
│  │  ┌────────────────────────┐  ┌────────────────────────┐                  │
│  │  │        Filters         │  │        Effects         │                  │
│  │  │ • Gaussian, Box Blur   │  │ • Grayscale, Sepia     │                  │
│  │  │ • Median, Sharpen      │  │ • Invert               │                  │
│  │  │ • Sobel, Canny         │  │ • Brightness, Contrast │                  │
│  │  └────────────────────────┘  └────────────────────────┘                  │
│  │                                                                          │
│  │  ┌────────────────────────┐  ┌────────────────────────┐                  │
│  │  │       Transform        │  │        Analysis        │                  │
│  │  │ • Resize (NN, Bilinear)│  │ • Histograms (Luma,    │                  │
│  │  │ • Rotate (Fixed, Arb)  │  │   RGB channels)        │                  │
│  │  │ • Mirror, Crop         │  │ • Stats (Min, Max, Avg)│                  │
│  │  └────────────────────────┘  └────────────────────────┘                  │
│  │                                                                          │
│  │  ┌────────────────────────┐  ┌────────────────────────┐                  │
│  │  │       Image I/O        │  │       Convolution      │                  │
│  │  │ • PNG, JPEG, BMP       │  │ • 2D Kernel            │                  │
│  │  │ • Raw Pixel Buffer     │  │ • Separable Kernel     │                  │
│  │  └────────────────────────┘  └────────────────────────┘                  │
│  └──────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Core Concepts (`silvestre-core`)

### 2.1 Image Buffer & Color Spaces

The foundational image data structure is `SilvestreImage`, which represents an owned, contiguous byte buffer stored in row-major order:

```rust
pub struct SilvestreImage {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    color_space: ColorSpace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Rgba,
    Rgb,
    Grayscale,
}
```

- **Immutability Principle:** Operations take `&self` and return a new `SilvestreImage` (`Result<SilvestreImage, SilvestreError>`), preventing unintended mutations and ensuring thread-safe composition.
- **Pixel Addressing:** Pixels are indexed by `(x, y)` coordinates where index = `(y * width + x) * channels`.

### 2.2 The `Filter` Trait

All image operations implement the unified `Filter` trait:

```rust
pub trait Filter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage, SilvestreError>;
}
```

This trait allows filters to be composed into execution chains and pipelines seamlessly.

### 2.3 Convolution Engine

Spatial filters use either general 2D kernels or optimized 1D separable kernels:
- **`Kernel`:** Standard $N \times M$ matrix convolution with border handling policies (`BorderHandling::Clamp`, `BorderHandling::Reflect`).
- **`SeparableKernel`:** Decomposes an $N \times N$ kernel into two $1 \times N$ passes (horizontal then vertical), reducing time complexity from $O(N^2)$ to $O(2N)$ per pixel (used by `GaussianFilter` and `BoxBlurFilter`).

---

## 3. Platform Adapters

### 3.1 `silvestre-cli` (Terminal Interface)
- Built with [Ratatui](https://ratatui.rs/) and [crossterm](https://crates.io/crates/crossterm).
- Provides an interactive Terminal User Interface (TUI) with half-block Unicode image rendering.
- Features multi-filter pipeline configuration, parameter sliders, live preview, and headless CLI subcommands (`apply`, `info`, `list`).

### 3.2 `silvestre-ffi` (C ABI Bindings)
- Exposes a stable, memory-safe C ABI using `extern "C"` functions.
- Generates C headers (`silvestre.h`) automatically using `cbindgen`.
- Implements explicit lifetime and memory management via opaque pointers (`*mut SilvestreImageHandle`) and `silvestre_image_free()`.
- Thread-local error reporting via `silvestre_last_error()`.

### 3.3 `silvestre-wasm` (WebAssembly)
- Uses `wasm-bindgen` to export web-ready image processing methods.
- Directly accepts and returns `Uint8ClampedArray` and browser `ImageData` byte slices.
- Includes a responsive web application (`www/`) with canvas preview and filter control sliders.

### 3.4 `silvestre_flutter` (Flutter Plugin)
- Bridges Rust core to Dart via [flutter_rust_bridge](https://cjycode.com/flutter_rust_bridge/) v2.
- Employs Flutter BLoC architecture for reactive state management.
- Example application provides real-time camera capture, interactive before/after split slider comparison, and device gallery export.

---

## 4. Error Handling & Safety

- **Core Error Handling:** Centralized via `SilvestreError` using the `thiserror` derive macro.
- **FFI Boundary:** Errors are mapped to negative integer status codes, preserving the descriptive error message in a thread-local buffer.
- **WASM Boundary:** Converted to JavaScript `JsValue` exception objects.
- **Flutter Bridge:** Auto-translated into Dart exceptions.
