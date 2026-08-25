# silvestre-core

[![crates.io](https://img.shields.io/crates/v/silvestre-core.svg)](https://crates.io/crates/silvestre-core)
[![docs.rs](https://docs.rs/silvestre-core/badge.svg)](https://docs.rs/silvestre-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

High-performance, pure-Rust image processing library providing composable spatial filters, color effects, geometric transformations, and histogram analysis.

`silvestre-core` has **zero platform dependencies**, making it ideal for desktop binaries, backend servers, embedded systems, WebAssembly, and mobile bridges.

---

## Installation

Add `silvestre-core` to your `Cargo.toml`:

```toml
[dependencies]
silvestre-core = "0.1"
```

Or using `cargo add`:

```bash
cargo add silvestre-core
```

---

## Features & Supported Operations

| Category | Algorithms & Operations |
|---|---|
| **Spatial & Convolution Filters** | Gaussian Blur, Box Blur, Median Filter, Sharpen, Sobel Edge Detection, Canny Edge Detection |
| **Color & Pixel Effects** | Grayscale (ITU-R BT.601), Sepia, Invert, Brightness, Contrast |
| **Geometric Transformations** | Resize (Nearest Neighbor & Bilinear Interpolation), Rotate (Fixed & Arbitrary angles), Mirror / Flip, Crop |
| **Image Analysis** | 256-bin Luminance & per-channel Histograms with statistics (Min, Max, Mean, Standard Deviation) |
| **Image I/O** | PNG, JPEG, BMP encoding and decoding via the [`image`](https://crates.io/crates/image) crate |

---

## Quick Start Example

Every filter implements the [`Filter`](https://docs.rs/silvestre-core/latest/silvestre_core/filters/trait.Filter.html) trait, taking `&SilvestreImage` and returning a newly allocated `SilvestreImage`.

```rust
use silvestre_core::effects::{BrightnessFilter, GrayscaleFilter};
use silvestre_core::filters::{CannyFilter, Filter, GaussianFilter};
use silvestre_core::transform::{ResizeFilter, RotateFilter};
use silvestre_core::analysis::Histogram;
use silvestre_core::SilvestreImage;

fn main() -> Result<(), silvestre_core::SilvestreError> {
    // 1. Load image from disk
    let image = SilvestreImage::load("input.png")?;

    // 2. Apply filters sequentially
    let blurred = GaussianFilter::new(2.0)?.apply(&image)?;
    let edges = CannyFilter::new(50.0, 150.0, 1.4).apply(&blurred)?;

    // 3. Apply geometric transformation
    let resized = ResizeFilter::bilinear(800, 600).apply(&image)?;
    let rotated = RotateFilter::new(45.0, 0, [0, 0, 0]).apply(&resized)?;

    // 4. Compute image histogram statistics
    if let Some(hist) = Histogram::luminance(&image) {
        let stats = hist.stats(0);
        println!("Mean intensity: {:.2}, StdDev: {:.2}", stats.mean, stats.std_dev);
    }

    // 5. Save the output
    edges.save("output_edges.png")?;
    rotated.save("output_rotated.png")?;

    Ok(())
}
```

---

## The `Filter` Trait

The library is centered around the composable `Filter` trait:

```rust
pub trait Filter: Send + Sync {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage, SilvestreError>;
}
```

Because filters do not mutate the input image in place, they can be safely shared across threads (`Send + Sync`) and composed into flexible execution pipelines:

```rust
let pipeline: Vec<Box<dyn Filter>> = vec![
    Box::new(GaussianFilter::new(1.5)?),
    Box::new(BrightnessFilter::new(20)),
    Box::new(GrayscaleFilter),
];

let mut current = image;
for filter in &pipeline {
    current = filter.apply(&current)?;
}
```

---

## Memory & Color Spaces

`SilvestreImage` holds an owned contiguous byte vector in row-major layout:

- **`ColorSpace::Grayscale`**: 1 byte per pixel.
- **`ColorSpace::Rgb`**: 3 bytes per pixel (R, G, B).
- **`ColorSpace::Rgba`**: 4 bytes per pixel (R, G, B, A).

Raw byte buffers can be accessed with `image.pixels()` or constructed via `SilvestreImage::new()`.

---

## License

MIT © [Enzo Lizama Paredes](https://github.com/enzoftware)
