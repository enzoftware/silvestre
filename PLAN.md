# Silvestre - Cross-Platform Image Processing Library

A Rust-based image processing library with multiplatform support: CLI, Android, iOS, Flutter, and WebAssembly.

Spiritual successor to the original Java/C# college project, rebuilt from scratch with modern architecture.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Platform Targets                       │
│                                                           │
│  ┌─────┐  ┌─────────┐  ┌──────┐  ┌─────┐  ┌──────────┐ │
│  │ CLI │  │ Flutter  │  │ iOS  │  │ Web │  │ Android  │ │
│  │     │  │(dart:ffi)│  │(C FFI│  │(WASM│  │ (JNI/    │ │
│  │     │  │  via     │  │  via │  │  via│  │  NDK)    │ │
│  │     │  │ flutter_ │  │ Uni- │  │wasm-│  │          │ │
│  │     │  │ rust_    │  │ FFI) │  │pack)│  │          │ │
│  │     │  │ bridge   │  │      │  │     │  │          │ │
│  └──┬──┘  └────┬─────┘  └──┬───┘  └──┬──┘  └────┬─────┘ │
│     │          │            │         │          │        │
│  ┌──┴──────────┴────────────┴─────────┴──────────┴─────┐ │
│  │              silvestre-ffi (C ABI layer)             │ │
│  │         Thin C-compatible wrapper over core          │ │
│  └─────────────────────┬───────────────────────────────┘ │
│                        │                                  │
│  ┌─────────────────────┴───────────────────────────────┐ │
│  │              silvestre-core (Pure Rust)              │ │
│  │                                                     │ │
│  │  ┌───────────┐ ┌───────────┐ ┌──────────────────┐  │ │
│  │  │  Filters  │ │  Effects  │ │  Transformations │  │ │
│  │  │           │ │           │ │                  │  │ │
│  │  │ • Canny   │ │ • Gray-   │ │ • Resize         │  │ │
│  │  │ • Median  │ │   scale   │ │ • Rotate          │  │ │
│  │  │ • Gauss-  │ │ • Sepia   │ │ • Mirror/Flip     │  │ │
│  │  │   ian     │ │ • Invert  │ │ • Crop            │  │ │
│  │  │ • Sobel   │ │ • Bright- │ │                  │  │ │
│  │  │ • Sharpen │ │   ness    │ │                  │  │ │
│  │  │ • Box     │ │ • Contrast│ │                  │  │ │
│  │  │   blur    │ │           │ │                  │  │ │
│  │  └───────────┘ └───────────┘ └──────────────────┘  │ │
│  │                                                     │ │
│  │  ┌───────────────┐  ┌───────────────────────────┐  │ │
│  │  │   Analysis    │  │       Image I/O           │  │ │
│  │  │               │  │                           │  │ │
│  │  │ • Histogram   │  │ • PNG, JPEG, BMP, WebP   │  │ │
│  │  │ • Stats       │  │ • Raw pixel buffer I/O   │  │ │
│  │  └───────────────┘  └───────────────────────────┘  │ │
│  └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

---

## Repository Structure

```
silvestre/
├── Cargo.toml                  # Workspace root
├── PLAN.md
├── README.md
│
├── crates/
│   ├── silvestre-core/         # Pure Rust image processing library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── image.rs        # Core image buffer type
│   │       ├── io.rs           # Image encoding/decoding
│   │       ├── filters/        # Convolution & spatial filters
│   │       │   ├── mod.rs
│   │       │   ├── canny.rs
│   │       │   ├── median.rs
│   │       │   ├── gaussian.rs
│   │       │   ├── sobel.rs
│   │       │   ├── sharpen.rs
│   │       │   └── box_blur.rs
│   │       ├── effects/        # Color/pixel-level effects
│   │       │   ├── mod.rs
│   │       │   ├── grayscale.rs
│   │       │   ├── sepia.rs
│   │       │   ├── invert.rs
│   │       │   ├── brightness.rs
│   │       │   └── contrast.rs
│   │       ├── transform/      # Geometric transformations
│   │       │   ├── mod.rs
│   │       │   ├── resize.rs
│   │       │   ├── rotate.rs
│   │       │   ├── mirror.rs
│   │       │   └── crop.rs
│   │       └── analysis/       # Image analysis tools
│   │           ├── mod.rs
│   │           └── histogram.rs
│   │
│   ├── silvestre-ffi/          # C-ABI foreign function interface
│   │   ├── Cargo.toml
│   │   ├── cbindgen.toml       # Auto-generates C headers
│   │   └── src/
│   │       └── lib.rs          # extern "C" functions
│   │
│   └── silvestre-cli/          # Command-line interface
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── silvestre-wasm/             # WebAssembly bindings
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs              # wasm-bindgen exports
│   └── www/                    # Demo web app
│       ├── index.html
│       └── index.js
│
├── silvestre-flutter/          # Flutter package with Rust FFI
│   ├── pubspec.yaml
│   ├── lib/
│   │   └── silvestre.dart
│   ├── rust/                   # flutter_rust_bridge Rust side
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── api.rs          # Public API for bridge
│   │       └── lib.rs
│   ├── android/
│   ├── ios/
│   ├── macos/
│   ├── linux/
│   ├── windows/
│   └── example/
│       └── ...                 # Flutter demo app
│
└── tests/
    ├── fixtures/               # Test images
    └── integration/            # Cross-crate integration tests
```

---

## Technology Stack

| Component | Technology |
|---|---|
| Core library | Rust (pure, no_std compatible where possible) |
| Image decoding | `image` crate (PNG, JPEG, BMP, WebP) |
| CLI | `clap` for arg parsing |
| C FFI | `cbindgen` for header generation |
| WASM | `wasm-bindgen` + `wasm-pack` |
| Flutter bridge | `flutter_rust_bridge` v2 |
| Android native | Rust → C ABI → JNI (via NDK) |
| iOS native | Rust → C ABI → Swift/ObjC interop |
| Testing | `cargo test` + property-based testing with `proptest` |
| CI | GitHub Actions (build all targets) |

---

## Implementation Phases

### Phase 1: Core Library Foundation
**Goal:** Working pure-Rust image processing library with tests.

- [ ] Set up Cargo workspace
- [ ] Implement `SilvestreImage` core type (pixel buffer, width, height, color space)
- [ ] Image I/O (load/save PNG, JPEG, BMP)
- [ ] Port original algorithms:
  - [ ] Median filter
  - [ ] Canny edge detection
  - [ ] Mirror/flip
  - [ ] Histogram computation
- [ ] Add new algorithms:
  - [ ] Grayscale conversion
  - [ ] Gaussian blur
  - [ ] Sobel edge detection
  - [ ] Brightness / Contrast adjustment
  - [ ] Sepia, Invert
  - [ ] Resize (nearest-neighbor + bilinear)
  - [ ] Rotate (90/180/270 + arbitrary angle)
  - [ ] Sharpen, Box blur
  - [ ] Crop
- [ ] Unit tests for every algorithm
- [ ] Benchmarks

### Phase 2: CLI Tool
**Goal:** Usable command-line tool for applying filters.

- [ ] `silvestre-cli` binary crate
- [ ] Commands: `apply <filter> --input <path> --output <path>`
- [ ] Pipeline support: chain multiple filters
- [ ] List available filters
- [ ] Progress output for large images

### Phase 3: C FFI Layer
**Goal:** Stable C ABI that all platform bindings can target.

- [ ] `silvestre-ffi` crate with `cdylib` + `staticlib` output
- [ ] C-compatible API:
  - `silvestre_image_load(path) -> *mut SilvestreImage`
  - `silvestre_image_from_buffer(data, len, w, h) -> *mut SilvestreImage`
  - `silvestre_apply_filter(img, filter_name, params) -> i32`
  - `silvestre_image_save(img, path, format) -> i32`
  - `silvestre_image_free(img)`
- [ ] `cbindgen` auto-generates `silvestre.h`
- [ ] Memory safety: all pointers validated, errors returned as codes

### Phase 4: WebAssembly
**Goal:** Run silvestre in the browser.

- [ ] `silvestre-wasm` crate with `wasm-bindgen`
- [ ] JS-friendly API (accepts/returns `Uint8Array`, `ImageData`)
- [ ] Demo web page with file upload + filter preview
- [ ] Build with `wasm-pack`
- [ ] Publish to npm (optional)

### Phase 5: Flutter Integration
**Goal:** Flutter package using `flutter_rust_bridge`.

- [ ] `silvestre-flutter` package scaffold
- [ ] Set up `flutter_rust_bridge` v2 codegen
- [ ] Dart API mirroring core:
  ```dart
  final image = await Silvestre.loadImage(path);
  final result = await image.applyFilter(SilvestreFilter.canny());
  await result.save('output.png');
  ```
- [ ] Android build (cargo-ndk → `.so` libraries)
- [ ] iOS build (cargo-lipo or xcode build phase → `.a` static lib)
- [ ] macOS, Linux, Windows desktop support
- [ ] Example Flutter app with camera + live filters
- [ ] Platform-specific build scripts in CI

### Phase 6: Polish & Release
**Goal:** Production-ready library.

- [ ] API documentation (`cargo doc`)
- [ ] README with examples for each platform
- [ ] GitHub Actions CI:
  - Build & test core on Linux/macOS/Windows
  - Build WASM
  - Build Flutter (Android + iOS)
- [ ] Publish `silvestre-core` to crates.io
- [ ] Publish `silvestre-flutter` to pub.dev
- [ ] Publish `silvestre-wasm` to npm
- [ ] Performance: SIMD acceleration for hot paths (optional)
- [ ] `no_std` support for embedded use (optional)

---

## Key Design Decisions

### 1. Image Buffer Type
The core image representation will be a simple owned pixel buffer:
```rust
pub struct SilvestreImage {
    pixels: Vec<u8>,     // RGBA, row-major
    width: u32,
    height: u32,
    color_space: ColorSpace,
}

pub enum ColorSpace {
    Rgba,
    Rgb,
    Grayscale,
}
```

### 2. Filter Trait
All filters implement a common trait for composability:
```rust
pub trait Filter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage, SilvestreError>;
}
```

### 3. FFI Memory Model
- Rust owns all image memory
- Foreign code receives opaque pointers
- Explicit `_free()` functions for deallocation
- No raw pixel pointer leaks across FFI boundary without a copy

### 4. Error Handling
- Core: `Result<T, SilvestreError>` with a rich error enum
- FFI: Integer error codes + `silvestre_last_error()` for messages
- WASM: JavaScript exceptions via `wasm-bindgen`
- Flutter: Dart exceptions via `flutter_rust_bridge`

---

## Dependencies (Rust)

```toml
# silvestre-core
[dependencies]
image = "0.25"        # Image decoding/encoding
thiserror = "2"       # Error derive macros

[dev-dependencies]
proptest = "1"         # Property-based testing
criterion = "0.5"      # Benchmarking

# silvestre-ffi
[dependencies]
silvestre-core = { path = "../silvestre-core" }

[build-dependencies]
cbindgen = "0.27"

# silvestre-cli
[dependencies]
silvestre-core = { path = "../silvestre-core" }
clap = { version = "4", features = ["derive"] }
indicatif = "0.17"     # Progress bars

# silvestre-wasm
[dependencies]
silvestre-core = { path = "../../crates/silvestre-core" }
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["ImageData"] }
```

---

## Getting Started (after scaffold)

```bash
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Run CLI
cargo run -p silvestre-cli -- apply median --input photo.jpg --output out.jpg

# Build WASM
cd silvestre-wasm && wasm-pack build --target web

# Build Flutter
cd silvestre-flutter && flutter_rust_bridge_codegen generate
flutter run
```

---

## Open Questions / Future Ideas

- **GPU acceleration**: Use `wgpu` for compute shader-based filters?
- **Streaming**: Process large images in tiles to reduce memory?
- **Plugin system**: Allow users to define custom filters via WASM modules?
- **Video**: Extend to video frame processing?
- **Python bindings**: Add PyO3 wrapper for the data science crowd?
