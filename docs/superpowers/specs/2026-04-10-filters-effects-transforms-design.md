# Filters, Effects, Transforms & Analysis Design Specification

**Date:** 2026-04-10  
**Topic:** Core Filters, Color Effects, Geometric Transforms, and Image Analysis  
**Pull Requests:** #45, #46, #47, #48, #49, #50, #51, #52, #53, #54, #56, #58  
**Status:** Implemented  

---

## 1. Overview

This specification details the algorithms, mathematical formulations, and component APIs for all image filters, effects, geometric transformations, and analysis utilities in `silvestre-core`.

---

## 2. Filters & Spatial Convolution (`src/filters/`)

### 2.1 Median Filter (`median.rs`)
- **Algorithm:** Non-linear spatial filter that replaces each pixel with the median value in an $N \times N$ neighborhood window.
- **Complexity:** $O(W \times H \times K^2 \log K)$ using quickselect or sorted array window.
- **Application:** Salt-and-pepper noise reduction while preserving sharp edge boundaries.

### 2.2 Gaussian Blur (`gaussian.rs`)
- **Kernel Computation:** 1D discrete Gaussian kernel:
  $$G(x) = \frac{1}{\sigma \sqrt{2\pi}} e^{-\frac{x^2}{2\sigma^2}}$$
- **Execution:** Separable 1D horizontal and vertical convolutions using `SeparableKernel`.

### 2.3 Sobel Edge Detection (`sobel.rs`)
- **Kernel Operators:**
  $$G_x = \begin{bmatrix} -1 & 0 & 1 \\ -2 & 0 & 2 \\ -1 & 0 & 1 \end{bmatrix}, \quad G_y = \begin{bmatrix} -1 & -2 & -1 \\ 0 & 0 & 0 \\ 1 & 2 & 1 \end{bmatrix}$$
- **Magnitude:** $G = \sqrt{G_x^2 + G_y^2}$ clamped to $[0, 255]$.

### 2.4 Canny Edge Detection (`canny.rs`)
Multi-stage edge detection pipeline:
1. **Gaussian Smoothing:** Reduce noise.
2. **Gradient Calculation:** Compute Sobel gradients and direction angles $\theta = \operatorname{atan2}(G_y, G_x)$.
3. **Non-Maximum Suppression (NMS):** Thin edges by preserving only local gradient maxima along the 4 quantized directions (0°, 45°, 90°, 135°).
4. **Double Thresholding:** Categorize pixels into strong, weak, and non-edge pixels using $T_{\text{high}}$ and $T_{\text{low}}$.
5. **Hysteresis Edge Tracking:** Retain weak pixels only if connected to strong edge pixels.

### 2.5 Box Blur & Sharpen (`box_blur.rs`, `sharpen.rs`)
- **Box Blur:** Uniform averaging kernel implemented via separable 1D passes.
- **Sharpen:** Unsharp masking / Laplacian enhancement kernel:
  $$\begin{bmatrix} 0 & -1 & 0 \\ -1 & 5 & -1 \\ 0 & -1 & 0 \end{bmatrix}$$

---

## 3. Pixel Effects (`src/effects/`)

### 3.1 Grayscale (`grayscale.rs`)
Luminance weighting according to ITU-R BT.601 standard:
$$Y = 0.299 R + 0.587 G + 0.114 B$$

### 3.2 Brightness & Contrast (`brightness.rs`, `contrast.rs`)
- **Brightness:** $P_{\text{out}} = \operatorname{clamp}(P_{\text{in}} + \Delta, 0, 255)$
- **Contrast:** $P_{\text{out}} = \operatorname{clamp}((P_{\text{in}} - 128) \times \text{factor} + 128, 0, 255)$

### 3.3 Sepia & Invert (`sepia.rs`, `invert.rs`)
- **Invert:** $P_{\text{out}} = 255 - P_{\text{in}}$ (alpha channel preserved).
- **Sepia Matrix:**
  $$\begin{aligned}
  R' &= \operatorname{clamp}(0.393R + 0.769G + 0.189B, 0, 255) \\
  G' &= \operatorname{clamp}(0.349R + 0.686G + 0.168B, 0, 255) \\
  B' &= \operatorname{clamp}(0.272R + 0.534G + 0.131B, 0, 255)
  \end{aligned}$$

---

## 4. Transformations (`src/transform/`)

### 4.1 Mirror (`mirror.rs`)
- Modes: `Horizontal`, `Vertical`, `Both`.

### 4.2 Resize (`resize.rs`)
- **Nearest Neighbor:** Fast mapping to nearest discrete pixel.
- **Bilinear Interpolation:** Smooth floating-point 2D linear blending across the 4 surrounding pixels.

### 4.3 Crop (`crop.rs`)
- Extracts rectangular sub-region `(x, y, width, height)` with boundary validation.

---

## 5. Image Analysis (`src/analysis/`)

### 5.1 Histogram & Intensity Statistics (`histogram.rs`)
- 256-bin histograms computed for luminance and individual color channels.
- Calculates statistical metrics: Minimum, Maximum, Mean, Median, and Standard Deviation.

---

## 6. Testing & Benchmarking

- Comprehensive unit tests across all color spaces.
- Property-based testing using `proptest`.
- Microbenchmarks using `criterion` for convolution and transformation hot paths.
