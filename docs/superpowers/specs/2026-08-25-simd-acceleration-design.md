# SIMD Acceleration Subsystem Design Specification

**Date:** 2026-08-25  
**Topic:** Hardware SIMD Acceleration, Multi-Architecture Dispatch, and Performance Optimization  
**Issue:** #39  
**Status:** Approved  

---

## 1. Overview

This design specification establishes the hardware SIMD (Single Instruction, Multiple Data) acceleration architecture for `silvestre-core`. It introduces architecture-optimized vectorized kernels for high-throughput image processing hot paths (color inversions, brightness adjustments, grayscale conversions, and 1D separable convolutions) across x86_64 (AVX2/SSE2), AArch64 (ARM NEON), and WebAssembly (WASM SIMD128), while guaranteeing strict bit-exact equivalence with portable scalar fallbacks.

---

## 2. Architecture & Module Design

### 2.1 Subsystem Layout (`silvestre-core/src/simd/`)

```text
silvestre-core/src/simd/
├── mod.rs             # Public entry points and architecture dispatchers
├── scalar.rs          # Portable auto-vectorized baseline & fallback implementation
├── aarch64.rs         # ARM NEON intrinsics (iOS, Android aarch64, Apple Silicon)
├── x86_64.rs          # x86_64 AVX2 & SSE2 intrinsics (dynamic runtime detection)
└── wasm.rs            # WebAssembly SIMD128 intrinsics (wasm32)
```

### 2.2 Target Architecture Dispatch Hierarchy

```text
               ┌───────────────────────────────┐
               │         SIMD Dispatch         │
               └───────────────┬───────────────┘
                               │
         ┌─────────────────────┼─────────────────────┐
         ▼                     ▼                     ▼
┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐
│  x86_64 Target  │   │ aarch64 Target  │   │  wasm32 Target  │
└────────┬────────┘   └────────┬────────┘   └────────┬────────┘
         │                     │                     │
    Is AVX2 CPU?          Use ARM NEON          Has SIMD128?
    ├── Yes: AVX2              │                     ├── Yes: v128
    └── No:  SSE2 /            ▼                     └── No:  Scalar
             Scalar         Scalar Fallback
```

1. **AArch64 (ARM NEON):**
   - Enabled by default on `target_arch = "aarch64"`.
   - Uses 128-bit vector registers (`uint8x16_t`, `float32x4_t`).
2. **x86_64 (AVX2 & SSE2):**
   - Dynamic runtime CPU feature detection via `is_x86_feature_detected!("avx2")` (256-bit `__m256i`, 32 bytes/cycle).
   - Graceful fallback to 128-bit SSE2 (`__m128i`, 16 bytes/cycle) or portable scalar loops.
3. **WebAssembly (`wasm32-unknown-unknown`):**
   - Compiled with `core::arch::wasm32` when `target_feature = "simd128"` is active, with fallback to scalar code.
4. **Scalar Baseline (`scalar.rs`):**
   - Portable, chunk-based loops structured to encourage compiler auto-vectorization on non-SIMD platforms.

---

## 3. Accelerated Operations & Intrinsics

### 3.1 Invert (`simd::invert`)
- **Grayscale / RGB:** Bitwise NOT on 16/32 bytes simultaneously:
  - NEON: `vmvnq_u8(v)`
  - AVX2: `_mm256_xor_si256(v, 255)`
  - WASM: `v128_not(v)`
- **RGBA:** Invert R, G, B channels while preserving Alpha (A):
  - Uses vector blend / mask operations (`vbslq_u8`, `_mm256_blendv_epi8`) to keep alpha unchanged without per-pixel branching.

### 3.2 Brightness Adjustment (`simd::brightness_add`, `simd::brightness_sub`)
- **Addition:** Saturated vector addition ($P + \Delta$ clamped to 255):
  - NEON: `vqaddq_u8(src, delta_vec)`
  - AVX2: `_mm256_adds_epu8(src, delta_vec)`
  - WASM: `u8x16_add_sat(src, delta_vec)`
- **Subtraction:** Saturated vector subtraction ($P - \Delta$ clamped to 0):
  - NEON: `vqsubq_u8(src, delta_vec)`
  - AVX2: `_mm256_subs_epu8(src, delta_vec)`
  - WASM: `u8x16_sub_sat(src, delta_vec)`
- **RGBA Alpha Channel:** Masked so alpha remains at its original value.

### 3.3 Grayscale BT.601 Conversion (`simd::grayscale_rgb`, `simd::grayscale_rgba`)
- **Fixed-Point Integer Weighting:**
  $$Y = (77 \cdot R + 150 \cdot G + 29 \cdot B + 128) \gg 8$$
  Exact mathematical approximation to $0.299R + 0.587G + 0.114B$.
- Processes batches of pixels in parallel using vector multiply-add and shift instructions (`vmlal_u8`, `_mm256_madd_epi16`).

---

## 4. Correctness & Tail Handling

SIMD operations divide input buffers into chunks of $V$ bytes (where $V = 32$ for AVX2 or $16$ for NEON/SSE2/WASM). Any remaining tail bytes ($N \pmod V \ne 0$) are processed by the scalar fallback, guaranteeing 100% correct behavior on arbitrary image widths and heights (including $1 \times 1$ and non-power-of-two dimensions).

---

## 5. Verification & Benchmarking Plan

1. **Unit & Equivalence Tests:**
   - Property tests asserting that `simd::invert`, `simd::brightness`, and `simd::grayscale` produce byte-identical results compared to the scalar baseline across all image sizes (1x1, 7x7, 100x100, 512x512, 1024x1024).
2. **Safety:**
   - All `unsafe` intrinsic blocks are encapsulated within `silvestre-core/src/simd/` with strict slice bounds checks on entry.
3. **Criterion Benchmarks:**
   - Profile throughput (MB/s) and latency on 100x100 and 512x512 images in `benches/filters.rs`.
   - Target speedup: **$\ge 2\times$** on supported hardware.
