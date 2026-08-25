# silvestre-wasm

[![npm version](https://img.shields.io/npm/v/silvestre-wasm.svg)](https://www.npmjs.com/package/silvestre-wasm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

High-performance WebAssembly bindings for the [silvestre](https://github.com/enzoftware/silvestre) image processing engine, powered by Rust and `wasm-bindgen`.

Load images, apply convolution filters, color effects, and transformations directly in the browser with zero server roundtrips at near-native speed.

---

## Installation

```bash
# Using npm
npm install silvestre-wasm

# Using yarn
yarn add silvestre-wasm

# Using pnpm
pnpm add silvestre-wasm
```

---

## Bundler Integration

### 1. Vite
Install `vite-plugin-wasm` and `vite-plugin-top-level-await`:

```ts
// vite.config.ts
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
});
```

### 2. Webpack 5
Enable WebAssembly experiments in your `webpack.config.js`:

```js
// webpack.config.js
module.exports = {
  experiments: {
    asyncWebAssembly: true,
    topLevelAwait: true,
  },
};
```

### 3. Next.js (App / Pages Router)
Enable async WebAssembly in `next.config.js`:

```js
// next.config.js
/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack: (config) => {
    config.experiments = { ...config.experiments, asyncWebAssembly: true };
    return config;
  },
};

module.exports = nextConfig;
```

---

## Usage Guide

### 1. Initialize the Module & Load Images

```ts
import init, { WasmImage } from "silvestre-wasm";

// Initialize WASM binary once at application startup
await init();

// Option A: Load from ArrayBuffer / Uint8Array (fetch or file input)
const response = await fetch("/photo.png");
const fileBytes = new Uint8Array(await response.arrayBuffer());
const image = WasmImage.loadFromBytes(fileBytes);

// Option B: Load directly from HTML5 Canvas ImageData
const canvas = document.querySelector<HTMLCanvasElement>("#myCanvas")!;
const ctx = canvas.getContext("2d")!;
const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
const imageFromCanvas = WasmImage.loadFromImageData(imageData);
```

### 2. Apply Filters & Transforms

Every operation returns a **new** `WasmImage` instance, allowing clean chaining without mutating the source image:

```ts
// 1. Effects
const grayscale = image.applyFilter("grayscale", {});
const sepia = image.applyFilter("sepia", {});
const invert = image.applyFilter("invert", {});
const bright = image.applyFilter("brightness", { delta: 40 });
const contrasted = image.applyFilter("contrast", { factor: 1.25 });

// 2. Convolution Filters
const blurred = image.applyFilter("gaussian", { sigma: 2.5 });
const sharp = image.applyFilter("sharpen", {});
const median = image.applyFilter("median", { size: 3 });
const edges = image.applyFilter("canny", { low: 50, high: 150, sigma: 1.4 });
const sobel = image.applyFilter("sobel", {});

// 3. Transformations
const resized = image.applyFilter("resize", { w: 800, h: 600 });
const rotated = image.applyFilter("rotate", { angle: 45 });
const flipped = image.applyFilter("mirror", { mode: "horizontal" });
const cropped = image.applyFilter("crop", { x: 50, y: 50, w: 400, h: 400 });

// 4. Chaining Pipelines
const result = image
  .applyFilter("gaussian", { sigma: 1.0 })
  .applyFilter("brightness", { delta: 15 })
  .applyFilter("contrast", { factor: 1.2 });
```

### 3. Rendering to Canvas & Exporting

```ts
// Render to HTML5 Canvas
const outImageData = result.toImageData();
canvas.width = outImageData.width;
canvas.height = outImageData.height;
ctx.putImageData(outImageData, 0, 0);

// Export encoded image bytes
const pngBlob = new Blob([result.toBytes("png")], { type: "image/png" });
const jpegBlob = new Blob([result.toBytes("jpeg")], { type: "image/jpeg" });
```

---

## API Reference

### `WasmImage` Class

| Method | Parameters | Returns | Description |
|---|---|---|---|
| `WasmImage.loadFromBytes(data)` | `data: Uint8Array` | `WasmImage` | Decodes PNG, JPEG, or BMP binary bytes into a WASM image buffer. |
| `WasmImage.loadFromImageData(data)` | `data: ImageData` | `WasmImage` | Constructs an image directly from raw RGBA canvas pixels. |
| `image.applyFilter(name, params)` | `name: string, params: object` | `WasmImage` | Applies an image operation and returns a newly allocated image. |
| `image.toImageData()` | — | `ImageData` | Returns browser-compatible RGBA `ImageData`. |
| `image.toBytes(format)` | `format: "png" \| "jpeg" \| "bmp"` | `Uint8Array` | Encodes the image into the specified compressed file format. |
| `image.width` | — | `number` | Width of the image in pixels. |
| `image.height` | — | `number` | Height of the image in pixels. |

---

## Building from Source

To compile the WebAssembly package locally:

```bash
# Build for npm bundlers (Webpack, Vite, Rollup)
wasm-pack build --target bundler --out-dir pkg

# Build for direct browser ES Module usage (without bundler)
wasm-pack build --target web --out-dir pkg-web
```

---

## License

MIT © [Enzo Lizama Paredes](https://github.com/enzoftware)
