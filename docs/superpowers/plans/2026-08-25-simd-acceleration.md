# SIMD Acceleration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement hardware SIMD acceleration for performance-critical image filters (Invert, Brightness, Grayscale) in `silvestre-core` across x86_64, AArch64/NEON, and WebAssembly SIMD128 with guaranteed scalar fallbacks.

**Architecture:** Create `silvestre-core/src/simd/` module with target-specific intrinsics and dynamic CPU feature dispatching, wire into `InvertFilter`, `BrightnessFilter`, and `GrayscaleFilter`, and verify via Criterion benchmarks.

**Tech Stack:** Rust (`core::arch`), Criterion, Proptest

**Spec:** `docs/superpowers/specs/2026-08-25-simd-acceleration-design.md`

## Global Constraints

- 100% stable Rust (no nightly feature flags required).
- Zero third-party dependencies added to `silvestre-core`.
- Bit-exact equivalence between SIMD paths and scalar baseline.
- Full support for arbitrary image dimensions and channel formats (Grayscale, RGB, RGBA).

---

### Task 1: SIMD Subsystem Scaffolding & Scalar Baseline
**Files:**
- Create: `silvestre-core/src/simd/mod.rs`
- Create: `silvestre-core/src/simd/scalar.rs`
- Modify: `silvestre-core/src/lib.rs`

- [x] **Step 1: Write `scalar.rs` implementing baseline operations (invert, brightness_add, brightness_sub, grayscale)**
- [x] **Step 2: Define public dispatcher interface in `simd/mod.rs`**
- [x] **Step 3: Add unit tests verifying scalar operations**
  - Run: `cargo test -p silvestre-core --lib simd`
  - Result: PASS (6 tests passed)

---

### Task 2: ARM NEON & AArch64 Vectorization
**Files:**
- Create: `silvestre-core/src/simd/aarch64.rs`
- Modify: `silvestre-core/src/simd/mod.rs`

- [x] **Step 1: Implement NEON vector intrinsics for `invert`, `brightness_add`, `brightness_sub`, and `grayscale` with 4x loop unrolling**
- [x] **Step 2: Implement tail processing using scalar fallback**
- [x] **Step 3: Add equivalence tests verifying NEON matches scalar output**
  - Run: `cargo test -p silvestre-core --lib simd`
  - Result: PASS

---

### Task 3: x86_64 AVX2 / SSE2 & WebAssembly SIMD128 Vectorization
**Files:**
- Create: `silvestre-core/src/simd/x86_64.rs`
- Create: `silvestre-core/src/simd/wasm.rs`
- Modify: `silvestre-core/src/simd/mod.rs`

- [x] **Step 1: Implement AVX2 (256-bit) and SSE2 (128-bit) vectorized loops with runtime `is_x86_feature_detected!`**
- [x] **Step 2: Implement WebAssembly SIMD128 intrinsics for browser targets**
- [x] **Step 3: Run full cross-target verification tests**
  - Run: `cargo test -p silvestre-core --lib simd`
  - Result: PASS

---

### Task 4: Filter Integration & Verification
**Files:**
- Modify: `silvestre-core/src/effects/invert.rs`
- Modify: `silvestre-core/src/effects/brightness.rs`
- Modify: `silvestre-core/src/effects/grayscale.rs`

- [x] **Step 1: Update `InvertFilter` to delegate pixel transformation to `simd::invert`**
- [x] **Step 2: Update `BrightnessFilter` to delegate to `simd::brightness_add` and `simd::brightness_sub`**
- [x] **Step 3: Update `GrayscaleFilter` to delegate to `simd::grayscale`**
- [x] **Step 4: Run all core and integration tests**
  - Run: `cargo test -p silvestre-core`
  - Result: PASS (316 unit tests, 13 integration tests, 7 property tests, 33 doctests)

---

### Task 5: Benchmarking & Documentation
**Files:**
- Modify: `silvestre-core/benches/filters.rs`
- Modify: `silvestre-core/README.md`
- Modify: `docs/architecture/overview.md`
- Modify: `docs/roadmap.md`

- [x] **Step 1: Add SIMD benchmarks in `benches/filters.rs`**
- [x] **Step 2: Measure benchmark throughput and speedup**
  - `invert_512x512`: **15.7 µs vs 224 µs (~14.3x speedup, −93% execution time)**
  - `grayscale_512x512`: **16.3 µs vs 70.6 µs (4.33x speedup)**
- [x] **Step 3: Update Superpowers plan, architecture overview, and roadmap with benchmark metrics**
