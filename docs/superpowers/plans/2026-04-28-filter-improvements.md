# Filter Improvements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve `BoxBlurFilter` and `SharpenFilter` by adding documentation and caching kernels for better performance and maintainability.

**Architecture:** 
- Add module-level and struct-level documentation with examples following the `GaussianFilter` style.
- Modify `BoxBlurFilter` and `SharpenFilter` structs to store their respective kernels.
- Initialize kernels during filter construction in `new` and `with_border`.
- Update `apply` methods to use the cached kernels.

**Tech Stack:** Rust, `silvestre-core`

---

## Task 1: Improve BoxBlurFilter

**Files:**
- Modify: `silvestre-core/src/filters/box_blur.rs`

- [ ] **Step 1: Update BoxBlurFilter struct and implementation**
  - Add documentation matching `GaussianFilter` style.
  - Add `kernel: SeparableKernel` field to `BoxBlurFilter`.
  - Update `new` and `with_border` to initialize the kernel.
  - Update `apply` to use the cached kernel.


- [ ] **Step 1: Run all filters tests**
  - Run: `cargo test -p silvestre-core --lib filters`
  - Expected: PASS

- [ ] **Step 2: Run all workspace tests**
  - Run: `cargo test`
  - Expected: PASS
