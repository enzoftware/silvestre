# Core Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the pure-Rust `silvestre-core` library containing foundational image representations, I/O encoders/decoders, the `Filter` trait, convolution engine, and initial filter/effect algorithms.

**Architecture:** Implement `SilvestreImage` with owned pixel storage in row-major layout, format encoding via the `image` crate, and generic 2D and separable 1D convolution pipelines.

**Tech Stack:** Rust (`cargo`), `image`, `thiserror`, `criterion`, `proptest`

**Spec:** `docs/superpowers/specs/2026-04-05-core-architecture-design.md`

## Global Constraints

- Pure Rust implementation with zero native or platform-specific dependencies.
- Memory safe: no unsafe blocks in core pixel manipulating code without explicit bounds checks.
- All public types and functions documented with example doctests.

---

### Task 1: Workspace & Core Buffer Type
**Files:**
- Create: `silvestre-core/Cargo.toml`
- Create: `silvestre-core/src/lib.rs`
- Create: `silvestre-core/src/image.rs`
- Create: `silvestre-core/src/error.rs`

- [x] **Step 1: Define `SilvestreImage` and `ColorSpace`**
- [x] **Step 2: Implement pixel buffer constructors, getters, and indexing**
- [x] **Step 3: Run unit tests**
  - Run: `cargo test -p silvestre-core --lib image`
  - Expected: PASS

---

### Task 2: Image I/O Integration
**Files:**
- Create: `silvestre-core/src/io.rs`

- [x] **Step 1: Implement `load` and `save` for PNG, JPEG, and BMP**
- [x] **Step 2: Run round-trip I/O tests**
  - Run: `cargo test -p silvestre-core --lib io`
  - Expected: PASS

---

### Task 3: Filter Trait & Convolution Engines
**Files:**
- Create: `silvestre-core/src/filters/mod.rs`
- Create: `silvestre-core/src/filters/convolution.rs`

- [x] **Step 1: Define `Filter` trait**
- [x] **Step 2: Implement `Kernel` (2D) and `SeparableKernel` (1D)**
- [x] **Step 3: Verify convolution equivalence and border clamping**
  - Run: `cargo test -p silvestre-core --lib filters`
  - Expected: PASS
