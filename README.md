# Silvestre

Cross-platform image processing library written in Rust.

Run the same filters natively on **CLI**, **Android**, **iOS**, **Flutter**, and **WebAssembly** from a single codebase.

> Spiritual successor to an old Java/C# college project, rebuilt from scratch with a modern Rust core.

## Architecture

```
 CLI    Flutter    iOS     Web    Android
  │    (dart:ffi)  (C FFI) (WASM) (JNI/NDK)
  │        │         │       │       │
  └────────┴─────────┴───────┴───────┘
                     │
            silvestre-ffi (C ABI)
                     │
            silvestre-core (Pure Rust)
```

**silvestre-core** contains all image processing logic as a pure Rust library. **silvestre-ffi** exposes it through a C ABI so every platform can consume it via its native FFI mechanism.

## Features

### Filters
Canny edge detection, Median, Gaussian blur, Sobel, Sharpen, Box blur

### Effects
Grayscale, Sepia, Invert, Brightness, Contrast

### Transforms
Resize (nearest-neighbor & bilinear), Rotate, Mirror/Flip, Crop

### Analysis
Histogram computation, Image statistics

### Image I/O
PNG, JPEG, BMP, WebP &mdash; load and save via the [`image`](https://crates.io/crates/image) crate

## Project Structure

```
silvestre/
├── crates/
│   ├── silvestre-core/    # Pure Rust image processing library
│   ├── silvestre-ffi/     # C ABI foreign function interface
│   └── silvestre-cli/     # Command-line tool
├── silvestre-wasm/        # WebAssembly bindings (planned)
├── silvestre-flutter/     # Flutter package via flutter_rust_bridge (planned)
└── tests/
    └── fixtures/          # Test images
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+

### Build

```bash
cargo build --workspace
```

### Test

```bash
cargo test --workspace
```

### CLI Usage

```bash
# Apply a filter
cargo run -p silvestre-cli -- apply median --input photo.jpg --output out.jpg

# List available filters
cargo run -p silvestre-cli -- list
```

## Rust API

```rust
use silvestre_core::{SilvestreImage, Filter};

// Load an image
let img = SilvestreImage::load("photo.png")?;

// Apply a filter (once implemented)
// let result = MedianFilter::new(3).apply(&img)?;

// Save the result
// result.save("output.png")?;
```

## Platform Targets

| Platform | Mechanism | Status |
|---|---|---|
| CLI | Native binary | In progress |
| WebAssembly | `wasm-bindgen` + `wasm-pack` | Planned |
| Flutter | `flutter_rust_bridge` v2 | Planned |
| Android | Rust &rarr; C ABI &rarr; JNI (NDK) | Planned |
| iOS | Rust &rarr; C ABI &rarr; Swift interop | Planned |

## Tech Stack

| Component | Technology |
|---|---|
| Core library | Rust |
| Image codec | [`image`](https://crates.io/crates/image) 0.25 |
| Error handling | [`thiserror`](https://crates.io/crates/thiserror) 2 |
| CLI | [`clap`](https://crates.io/crates/clap) 4 |
| C header gen | [`cbindgen`](https://crates.io/crates/cbindgen) |
| WASM | `wasm-bindgen` |
| Flutter bridge | `flutter_rust_bridge` v2 |
| Testing | `cargo test` + `proptest` |
| Benchmarks | `criterion` |

## Roadmap

See [PLAN.md](PLAN.md) for the full implementation plan.

1. **Core library** &mdash; image types, filters, effects, transforms, analysis
2. **CLI tool** &mdash; apply filters from the command line
3. **C FFI layer** &mdash; stable ABI for platform bindings
4. **WebAssembly** &mdash; run in the browser
5. **Flutter package** &mdash; Android, iOS, and desktop via `flutter_rust_bridge`
6. **Polish & release** &mdash; CI, docs, publish to crates.io / pub.dev / npm

## License

MIT
