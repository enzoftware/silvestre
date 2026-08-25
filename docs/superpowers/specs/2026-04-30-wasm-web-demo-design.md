# WebAssembly Bindings & Web Demo Design Specification

**Date:** 2026-04-30  
**Topic:** WebAssembly Crate, Canvas ImageData Interop, and Web Demo  
**Pull Requests:** #60  
**Status:** Implemented  

---

## 1. Overview

This specification establishes `silvestre-wasm`, enabling the Silvestre image processing engine to execute client-side inside modern web browsers at near-native performance using WebAssembly and `wasm-bindgen`.

---

## 2. Architecture & Bindings Design

### 2.1 WebAssembly Export Interface (`silvestre-wasm/src/lib.rs`)

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmImage {
    inner: SilvestreImage,
}

#[wasm_bindgen]
impl WasmImage {
    #[wasm_bindgen(constructor)]
    pub fn from_raw_pixels(pixels: &[u8], width: u32, height: u32) -> Result<WasmImage, JsValue>;

    #[wasm_bindgen]
    pub fn to_raw_pixels(&self) -> Vec<u8>;

    #[wasm_bindgen]
    pub fn apply_gaussian(&self, sigma: f32) -> Result<WasmImage, JsValue>;

    #[wasm_bindgen]
    pub fn apply_canny(&self, low: f32, high: f32) -> Result<WasmImage, JsValue>;

    #[wasm_bindgen]
    pub fn apply_grayscale(&self) -> Result<WasmImage, JsValue>;

    #[wasm_bindgen]
    pub fn apply_sepia(&self) -> Result<WasmImage, JsValue>;

    #[wasm_bindgen]
    pub fn apply_invert(&self) -> Result<WasmImage, JsValue>;
}
```

### 2.2 Browser Canvas Interop
- Direct mapping between HTML5 `<canvas>` `ImageData.data` (`Uint8ClampedArray` containing 4-channel RGBA) and Rust's `SilvestreImage`.
- Minimal copy overhead when transferring pixel buffers between browser JavaScript runtime and WASM linear memory.

---

## 3. Web Demo Application (`silvestre-wasm/www/`)

- Modern web interface (`index.html`, `index.js`, CSS) demonstrating real-time browser-based processing.
- Features drag-and-drop image file upload, filter selection dropdown, parameter sliders with debounced execution, and instantaneous before/after canvas rendering.

---

## 4. Build Pipeline & Verification

- Compiles using `wasm-pack build --target web`.
- Verified via `wasm-pack test --headless --chrome` and interactive browser tests.
