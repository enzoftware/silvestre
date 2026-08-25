# Silvestre

[![CI](https://github.com/enzoftware/silvestre/actions/workflows/ci.yml/badge.svg)](https://github.com/enzoftware/silvestre/actions/workflows/ci.yml)

Cross-platform image processing library written in Rust.

Run the same filters natively on the **CLI**, on **Android** and **iOS** via
**Flutter**, and in the **browser** via **WebAssembly** — all from a single pure
Rust core.

> Spiritual successor to an old Java/C# college project, rebuilt from scratch
> with a modern Rust core.

## Architecture

```text
   CLI        Flutter          Web          C / C++ / Swift / JNI
 (native)   (dart:ffi)        (WASM)              (C ABI)
    │            │               │                    │
    │      silvestre_flutter  silvestre-wasm    silvestre-ffi
    │        (frb bridge)    (wasm-bindgen)      (C ABI, cbindgen)
    │            │               │                    │
    └────────────┴───────────────┴────────────────────┘
                             │
                    silvestre-core  (pure Rust, no platform deps)
```

**silvestre-core** holds every image-processing operation as a pure Rust
library with no platform dependencies. Each platform crate is a thin binding
layer over it:

- **silvestre-cli** — an interactive terminal UI (ratatui).
- **silvestre-ffi** — a stable C ABI (with a `cbindgen`-generated header) for
  C/C++, Swift, and JNI consumers.
- **silvestre-wasm** — `wasm-bindgen` bindings for the browser.
- **silvestre_flutter** — a Flutter plugin using `flutter_rust_bridge`.

## Features

| Category | Operations |
|---|---|
| **Filters** | Box blur, Gaussian blur, Median, Sharpen, Sobel, Canny edge detection |
| **Effects** | Grayscale, Sepia, Invert, Brightness, Contrast |
| **Transforms** | Resize (nearest-neighbor & bilinear), Rotate, Mirror/Flip, Crop |
| **Analysis** | Per-channel & luminance histograms with statistics |
| **Image I/O** | Load & save PNG, JPEG, BMP via the [`image`](https://crates.io/crates/image) crate |

## Project structure

```text
silvestre/
├── silvestre-core/      # Pure Rust image processing library
├── silvestre-ffi/       # C ABI foreign function interface (cbindgen header)
├── silvestre-cli/       # Interactive terminal UI (ratatui)
├── silvestre-wasm/      # WebAssembly bindings (wasm-bindgen) + web demo
├── silvestre_flutter/   # Flutter plugin via flutter_rust_bridge + example app
├── docs/                # Architecture docs, roadmap, and Superpowers specs/plans
│   ├── architecture/    # Deep-dive system architecture documentation
│   ├── roadmap.md       # Milestone tracker & upcoming roadmap
│   └── superpowers/     # Design specifications (specs/) & implementation plans (plans/)
├── AGENTS.md            # Universal contributor & agent guidance
└── tests/fixtures/      # Test images
```

## Documentation

- **[Architecture Overview](docs/architecture/overview.md)** — Detailed multi-target topology, convolution engine, memory models, and crate interactions.
- **[Roadmap & Milestone Tracker](docs/roadmap.md)** — Completed phases (PRs #41–#65) and upcoming features (GPU compute, tiled streaming, WASM plugins).
- **[Superpowers Specifications](docs/superpowers/specs/)** — Design specs for all milestones and subsystem features.
- **[Superpowers Implementation Plans](docs/superpowers/plans/)** — Step-by-step TDD implementation plans.
- **[Agent Guidelines](AGENTS.md)** — Development standards, commit conventions, and testing commands.

## Quick start

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+ (`cargo`, `rustc`)
- Platform extras, as needed:
  - WASM: `rustup target add wasm32-unknown-unknown` and
    [wasm-pack](https://rustwasm.github.io/wasm-pack/)
  - Flutter: the [Flutter SDK](https://docs.flutter.dev/get-started/install)

Build and test the whole workspace:

```bash
cargo build --workspace
cargo test --workspace
```

---

### 1. Rust (`silvestre-core`)

Available on crates.io as [`silvestre-core`](https://crates.io/crates/silvestre-core):

```toml
[dependencies]
silvestre-core = "0.1"
```

Or install with `cargo`:

```bash
cargo add silvestre-core
```

Every operation implements the [`Filter`] trait and returns a **new** image,
leaving the input untouched, so filters compose cleanly:

```rust
use silvestre_core::effects::{BrightnessFilter, GrayscaleFilter};
use silvestre_core::filters::{Filter, GaussianFilter};
use silvestre_core::SilvestreImage;

fn main() -> Result<(), silvestre_core::SilvestreError> {
    // Load from disk (PNG, JPEG, or BMP).
    let image = SilvestreImage::load("photo.png")?;

    // Apply filters in sequence.
    let result = GrayscaleFilter.apply(&image)?;
    let result = BrightnessFilter::new(30).apply(&result)?;
    let result = GaussianFilter::new(2.0)?.apply(&result)?;

    // Save the result (format inferred from the extension).
    result.save("output.png")?;
    Ok(())
}
```

Full API docs: `cargo doc --no-deps -p silvestre-core --open`.

---

### 2. CLI (`silvestre-cli`)

An interactive terminal UI for browsing filters and building filter pipelines:

```bash
cargo run -p silvestre-cli
```

Use the arrow keys to navigate, `Space` to toggle filters in the pipeline, and
follow the on-screen hints to load an image and export the result.

---

### 3. WebAssembly (`silvestre-wasm`)

Available on npm as [`silvestre-wasm`](https://www.npmjs.com/package/silvestre-wasm):

```bash
npm install silvestre-wasm
# or build locally with wasm-pack:
cd silvestre-wasm && wasm-pack build --target bundler --out-dir pkg
```

Use it from JavaScript or TypeScript with modern bundlers (Vite, Webpack 5, Next.js):

```ts
import init, { WasmImage } from "silvestre-wasm";

await init(); // call once before using WasmImage

// Load from file bytes (e.g. a fetch or <input type="file">).
const bytes = new Uint8Array(await (await fetch("/photo.png")).arrayBuffer());
const image = WasmImage.loadFromBytes(bytes);

// Filters chain; each call returns a new WasmImage.
const result = image
  .applyFilter("grayscale", {})
  .applyFilter("brightness", { delta: 20 })
  .applyFilter("gaussian", { sigma: 2.0 });

// Render to a <canvas>…
const canvas = document.querySelector<HTMLCanvasElement>("#preview");
const context = canvas?.getContext("2d");
if (!context) throw new Error("2D canvas context unavailable");
const imageData = result.toImageData();
context.putImageData(imageData, 0, 0);

// …or export encoded bytes.
const png = result.toBytes("png"); // Uint8Array
```

A complete Vite demo lives in [`silvestre-wasm/www`](silvestre-wasm/www). See the
[crate README](silvestre-wasm/README.md) for the full JS API and bundler configs.

---

### 4. Flutter (`silvestre_flutter`)

Available on pub.dev as [`silvestre_flutter`](https://pub.dev/packages/silvestre_flutter):

```bash
flutter pub add silvestre_flutter
```

Call `Silvestre.init()` once before using the API. Every operation runs in Rust on a background isolate, so the async Dart API keeps the UI responsive:

```dart
import 'package:flutter/widgets.dart';
import 'package:silvestre_flutter/silvestre_flutter.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await Silvestre.init();

  final image = await SilvestreImage.fromPath('/path/to/photo.png');

  // Convenience methods…
  final gray = await image.grayscale();
  // …or the generic entry point with snake_case names + params.
  final blurred = await image.applyFilter('gaussian', params: {'sigma': 2.0});

  await blurred.save('/path/to/output.png');
}
```

The plugin ships a full Bloc-based example app (camera capture, gallery save,
live histogram, before/after slider) in
[`silvestre_flutter/example`](silvestre_flutter/example). See the
[plugin README](silvestre_flutter/README.md) for the full Dart API.

---

### 5. C ABI (`silvestre-ffi`)

Building the crate regenerates the C header at
[`silvestre-ffi/include/silvestre.h`](silvestre-ffi/include/silvestre.h) via
`cbindgen`:

```bash
cargo build -p silvestre-ffi   # produces libsilvestre_ffi + silvestre.h
```

Consume it from C (or any language with C interop — Swift, JNI, etc.):

```c
#include <stdio.h>
#include "silvestre.h"

SilvestreImage *img = silvestre_image_load("photo.png");
if (img == NULL) {
    fprintf(stderr, "load failed: %s\n", silvestre_last_error());
    return 1;
}

// Apply a filter by name; parameters are a JSON string (or NULL for none).
if (silvestre_apply_filter(img, "gaussian", "{\"sigma\": 2.0}") != 0) {
    fprintf(stderr, "filter failed: %s\n", silvestre_last_error());
    silvestre_image_free(img);
    return 1;
}

// NULL format → infer from the file extension.
if (silvestre_image_save(img, "output.png", NULL) != 0) {
    fprintf(stderr, "save failed: %s\n", silvestre_last_error());
    silvestre_image_free(img);
    return 1;
}

silvestre_image_free(img);
```

Error handling is thread-local: functions return `0` on success and `-1` on
error; call `silvestre_last_error()` for the message.

## Documentation

Generate the full Rust API documentation for the workspace:

```bash
cargo doc --no-deps --workspace --open
```

All public items are documented, and inline examples are verified by
`cargo test --doc`.

## Contributing

Contributions are welcome! To get started:

1. **Fork and branch.** Create a feature branch from `main`
   (e.g. `feat/my-filter`).
2. **Keep the core pure.** New image-processing logic belongs in
   `silvestre-core` behind the [`Filter`] trait, not in a platform crate.
   Platform crates should only adapt types and delegate to the core.
3. **Document public items.** Every crate enforces `#![warn(missing_docs)]`;
   add a `///` doc comment (with a `# Examples` block where it helps) to any new
   public type or function.
4. **Test thoroughly.** Cover happy paths, edge cases, and error conditions.
   Doc examples double as tests.
5. **Verify before opening a PR:**

   ```bash
   cargo build --workspace
   cargo test --workspace
   cargo test --doc
   cargo doc --no-deps --workspace   # must be warning-free
   cargo fmt --all
   cargo clippy --workspace -- -D warnings
   ```

6. **Open a PR** describing the change and referencing any related issue.

## Tech stack

| Component | Technology |
|---|---|
| Core library | Rust |
| Image codec | [`image`](https://crates.io/crates/image) 0.25 |
| Error handling | [`thiserror`](https://crates.io/crates/thiserror) 2 |
| CLI | [`ratatui`](https://crates.io/crates/ratatui) + [`crossterm`](https://crates.io/crates/crossterm) |
| C header gen | [`cbindgen`](https://crates.io/crates/cbindgen) |
| WASM | [`wasm-bindgen`](https://crates.io/crates/wasm-bindgen) + `wasm-pack` |
| Flutter bridge | [`flutter_rust_bridge`](https://pub.dev/packages/flutter_rust_bridge) v2 |

## License

MIT
