# silvestre-wasm

WebAssembly bindings for the [silvestre](https://github.com/enzoftware/silvestre) image processing library, powered by `wasm-bindgen`.

Load images, apply filters, and render results — all in the browser with no server round-trips.

## Prerequisites

- [Rust](https://rustup.rs/) with the `wasm32-unknown-unknown` target
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (`cargo install wasm-pack`)
- [Node.js](https://nodejs.org/) (for the demo app)

## Build

```bash
# From the silvestre-wasm directory:
wasm-pack build --target web --out-dir pkg
```

This produces a `pkg/` directory with the `.wasm` binary, JS glue code, and TypeScript declarations.

## Usage

### Initialize the WASM module

```ts
import init, { WasmImage } from "silvestre-wasm";

await init(); // must be called once before using WasmImage
```

### Load an image

```ts
// From file bytes (e.g. from a fetch or file input)
const response = await fetch("/photo.png");
const bytes = new Uint8Array(await response.arrayBuffer());
const image = WasmImage.loadFromBytes(bytes);

// From canvas ImageData
const canvas = document.querySelector("canvas");
const ctx = canvas.getContext("2d");
const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
const image = WasmImage.loadFromImageData(imageData);
```

### Apply filters

```ts
// No-param filters
const gray = image.applyFilter("grayscale", {});

// Filters with parameters
const bright = image.applyFilter("brightness", { delta: 50 });
const blurred = image.applyFilter("gaussian", { sigma: 2.0 });
const edges = image.applyFilter("canny", { low: 50, high: 100, sigma: 1.4 });

// Transforms
const resized = image.applyFilter("resize", { w: 800, h: 600 });
const rotated = image.applyFilter("rotate", { angle: 90 });
const mirrored = image.applyFilter("mirror", { mode: "horizontal" });
const cropped = image.applyFilter("crop", { x: 10, y: 10, w: 200, h: 200 });

// Chain filters (each returns a new WasmImage)
const result = image
  .applyFilter("brightness", { delta: 20 })
  .applyFilter("contrast", { factor: 1.3 })
  .applyFilter("sharpen", {});
```

### Render to canvas

```ts
const imageData = image.toImageData();
const canvas = document.querySelector("canvas");
canvas.width = imageData.width;
canvas.height = imageData.height;
canvas.getContext("2d").putImageData(imageData, 0, 0);
```

### Export

```ts
const pngBytes = image.toBytes("png");   // Uint8Array
const jpgBytes = image.toBytes("jpeg");
const bmpBytes = image.toBytes("bmp");
```

## API Reference

### `WasmImage`

| Method / Property | Description |
|---|---|
| `WasmImage.loadFromBytes(data: Uint8Array)` | Load from PNG/JPEG/BMP bytes |
| `WasmImage.loadFromImageData(data: ImageData)` | Load from canvas ImageData |
| `image.applyFilter(name: string, params: object)` | Apply a filter, returns new WasmImage |
| `image.toImageData()` | Convert to RGBA ImageData |
| `image.toBytes(format: string)` | Encode to "png", "jpeg", or "bmp" |
| `image.width` | Image width in pixels |
| `image.height` | Image height in pixels |

### Available Filters

| Filter | Params | Category |
|---|---|---|
| `grayscale` | — | Effects |
| `invert` | — | Effects |
| `sepia` | — | Effects |
| `brightness` | `delta: i32` | Effects |
| `contrast` | `factor: f32` | Effects |
| `sharpen` | — | Filters |
| `box_blur` | — | Filters |
| `sobel` | — | Filters |
| `gaussian` | `sigma: f32` | Filters |
| `median` | `size: usize` (odd) | Filters |
| `canny` | `low: f32`, `high: f32`, `sigma: f32` | Filters |
| `resize` | `w: u32`, `h: u32` | Transforms |
| `rotate` | `angle: f64` (degrees) | Transforms |
| `mirror` | `mode: "horizontal"\|"vertical"\|"both"` | Transforms |
| `crop` | `x: u32`, `y: u32`, `w: u32`, `h: u32` | Transforms |

## Demo App

An interactive demo built with Vite + React + TypeScript + shadcn/ui is included in `www/`.

```bash
cd www
npm install
npm run dev
```

Features:
- Drag & drop image upload (PNG, JPEG, BMP)
- Side-by-side original/result comparison
- All 15 filters with interactive parameter controls
- Loading states during WASM initialization and filter processing
- Download processed images as PNG
