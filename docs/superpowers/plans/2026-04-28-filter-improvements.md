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

### Task 1: Improve BoxBlurFilter

**Files:**
- Modify: `silvestre-core/src/filters/box_blur.rs`

- [ ] **Step 1: Update BoxBlurFilter struct and implementation**
  - Add documentation matching `GaussianFilter` style.
  - Add `kernel: SeparableKernel` field to `BoxBlurFilter`.
  - Update `new` and `with_border` to initialize the kernel.
  - Update `apply` to use the cached kernel.

```rust
//! Box blur filter.
//!
//! Applies a simple box blur using a separable 3x3 kernel.

use crate::filters::convolution::{apply_separable_kernel, BorderMode, SeparableKernel};
use crate::filters::Filter;
use crate::{Result, SilvestreImage};

/// Box blur filter with configurable border mode.
///
/// Applies a 3x3 box blur where each pixel is the average of its 3x3 neighborhood.
/// Uses a separable kernel for efficiency.
///
/// # Examples
///
/// ```
/// use silvestre_core::filters::{Filter, BoxBlurFilter};
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let img = SilvestreImage::new(vec![100; 25], 5, 5, ColorSpace::Grayscale)?;
/// let blur = BoxBlurFilter::new();
/// let out = blur.apply(&img)?;
/// assert_eq!(out.width(), 5);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone)]
pub struct BoxBlurFilter {
    kernel: SeparableKernel,
    border: BorderMode,
}

impl BoxBlurFilter {
    /// Create a new box blur filter with default border mode (Clamp).
    pub fn new() -> Result<Self> {
        Self::with_border(BorderMode::Clamp)
    }

    /// Create a new box blur filter with a specific border mode.
    pub fn with_border(border: BorderMode) -> Result<Self> {
        let coeffs = vec![1.0 / 3.0; 3];
        let kernel = SeparableKernel::new(coeffs.clone(), coeffs)?;
        Ok(Self { kernel, border })
    }
}

impl Filter for BoxBlurFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        apply_separable_kernel(image, &self.kernel, self.border)
    }
}
```

- [ ] **Step 2: Update tests in box_blur.rs**
  - Update test cases to handle `Result` from `BoxBlurFilter::new()`.

- [ ] **Step 3: Run tests and verify**
  - Run: `cargo test -p silvestre-core --lib filters::box_blur`
  - Expected: PASS

- [ ] **Step 4: Commit BoxBlurFilter improvements**

```bash
git add silvestre-core/src/filters/box_blur.rs
git commit -m "refactor: add docs and cache kernel for BoxBlurFilter"
```

---

### Task 2: Improve SharpenFilter

**Files:**
- Modify: `silvestre-core/src/filters/sharpen.rs`

- [ ] **Step 1: Update SharpenFilter struct and implementation**
  - Add documentation matching `GaussianFilter` style.
  - Add `kernel: Kernel` field to `SharpenFilter`.
  - Update `new` and `with_border` to initialize the kernel.
  - Update `apply` to use the cached kernel.

```rust
//! Sharpen filter.
//!
//! Enhances edges in an image using a Laplacian-based kernel.

use crate::filters::convolution::{apply_kernel, BorderMode, Kernel};
use crate::filters::Filter;
use crate::{Result, SilvestreImage};

/// Sharpen filter using a Laplacian-based 3x3 kernel.
///
/// This filter enhances edges by subtracting the Laplacian (second derivative)
/// from the original image.
///
/// # Examples
///
/// ```
/// use silvestre_core::filters::{Filter, SharpenFilter};
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let img = SilvestreImage::new(vec![100; 25], 5, 5, ColorSpace::Grayscale)?;
/// let filter = SharpenFilter::new();
/// let out = filter.apply(&img)?;
/// assert_eq!(out.width(), 5);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone)]
pub struct SharpenFilter {
    kernel: Kernel,
    border: BorderMode,
}

impl SharpenFilter {
    /// Create a new sharpen filter with default border mode (Clamp).
    pub fn new() -> Result<Self> {
        Self::with_border(BorderMode::Clamp)
    }

    /// Create a new sharpen filter with a specific border mode.
    pub fn with_border(border: BorderMode) -> Result<Self> {
        let kernel = Kernel::square(
            vec![
                 0.0, -1.0,  0.0,
                -1.0,  5.0, -1.0,
                 0.0, -1.0,  0.0,
            ],
            3,
        )?;
        Ok(Self { kernel, border })
    }
}

impl Filter for SharpenFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        apply_kernel(image, &self.kernel, self.border)
    }
}
```

- [ ] **Step 2: Update tests in sharpen.rs**
  - Update test cases to handle `Result` from `SharpenFilter::new()`.

- [ ] **Step 3: Run tests and verify**
  - Run: `cargo test -p silvestre-core --lib filters::sharpen`
  - Expected: PASS

- [ ] **Step 4: Commit SharpenFilter improvements**

```bash
git add silvestre-core/src/filters/sharpen.rs
git commit -m "refactor: add docs and cache kernel for SharpenFilter"
```

---

### Task 3: Final Verification

- [ ] **Step 1: Run all filters tests**
  - Run: `cargo test -p silvestre-core --lib filters`
  - Expected: PASS

- [ ] **Step 2: Run all workspace tests**
  - Run: `cargo test`
  - Expected: PASS
