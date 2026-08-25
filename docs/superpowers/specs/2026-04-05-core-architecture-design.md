# Core Architecture & Workspace Foundation Design Specification

**Date:** 2026-04-05  
**Topic:** Core Workspace, SilvestreImage, I/O & Filter Infrastructure  
**Pull Requests:** #41, #42, #43, #44  
**Status:** Implemented  

---

## 1. Overview

This specification establishes the pure Rust foundational library `silvestre-core` and the Cargo workspace topology for the Silvestre project. The goal is to build a memory-safe, high-performance image processing kernel that operates independently of any platform-specific dependencies.

---

## 2. Architecture & Module Design

### 2.1 Cargo Workspace Layout
The repository is configured as a multi-crate Cargo workspace:
- `silvestre-core`: Pure Rust image manipulation engine.
- `silvestre-ffi`: C ABI export wrapper.
- `silvestre-cli`: Terminal command-line and TUI binary.
- `silvestre-wasm`: WebAssembly browser module.

### 2.2 Core Data Structures (`silvestre-core/src/image.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
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

- **Pixel Storage:** Flat, contiguous `Vec<u8>` in row-major layout.
- **Channels:**
  - `Grayscale`: 1 byte per pixel.
  - `Rgb`: 3 bytes per pixel (Red, Green, Blue).
  - `Rgba`: 4 bytes per pixel (Red, Green, Blue, Alpha).
- **Invariants:** `pixels.len() == (width * height * channels) as usize`.

### 2.3 Image I/O (`silvestre-core/src/io.rs`)
Image encoding and decoding is implemented using the `image` crate:
- `SilvestreImage::load(path: impl AsRef<Path>) -> Result<SilvestreImage, SilvestreError>`
- `SilvestreImage::save(&self, path: impl AsRef<Path>) -> Result<(), SilvestreError>`
- Format autodetection from file extension (supporting PNG, JPEG, BMP).

### 2.4 The `Filter` Trait & Convolution (`silvestre-core/src/filters/`)

```rust
pub trait Filter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage, SilvestreError>;
}
```

- **2D Kernel Convolution (`Kernel`):**
  - Symmetric or asymmetric 2D convolution matrix with configurable normalization divisor.
  - Border policies: `Clamp` (repeat nearest boundary pixel) and `Reflect`.
- **1D Separable Convolution (`SeparableKernel`):**
  - Allows 2D separable filters (e.g. Gaussian, Box Blur) to be executed in two 1D passes (horizontal pass followed by vertical pass), reducing computations from $O(W \times H \times K^2)$ to $O(W \times H \times 2K)$.

---

## 3. Error Handling (`silvestre-core/src/error.rs`)

Using `thiserror`:
```rust
#[derive(Debug, thiserror::Error)]
pub enum SilvestreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image format error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Invalid dimensions: width={width}, height={height}")]
    InvalidDimensions { width: u32, height: u32 },
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}
```

---

## 4. Verification & Testing

- Unit tests for image instantiation, dimension validation, pixel indexing, and color space conversions.
- Round-trip image encoding and decoding tests for PNG, JPEG, and BMP.
- Equivalence verification between 2D convolution and separable 1D convolution.
