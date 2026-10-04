---
name: implement-effect
description: >
  Step-by-step workflow for adding a new image effect or filter to silvestre-core.
  Covers module placement, the Filter trait implementation pattern, scalar + SIMD
  fast-paths, test checklist, and post-implementation verification gates.
  Invoke with the effect name and type: /implement-effect <EffectName> [effect|filter|transform]
---

# implement-effect — Add a New Effect to silvestre-core

## Argument parsing

Parse `$ARGUMENTS` as: `<EffectName> [module-type]`

- `EffectName` — PascalCase struct name, e.g. `HueSaturation`, `Vignette`, `UnsharpMask`.
- `module-type` — one of `effect`, `filter`, or `transform` (default: `effect`).

Derive the snake_case file name automatically: `HueSaturation` → `hue_saturation`.

---

## Pre-flight Validation

**Run these checks BEFORE writing any code. Stop and report if any check fails.**

```bash
# 1. Confirm the workspace builds cleanly on entry
cargo build --workspace --quiet

# 2. Confirm all tests pass on entry (establish a green baseline)
cargo test --workspace --quiet

# 3. Confirm the effect does not already exist
grep -r "<SnakeCaseName>" silvestre-core/src/ --include="*.rs" -l
# Expected: no output. If output appears, the effect already exists — abort.

# 4. Confirm the struct name doesn't collide in lib.rs exports
grep "<EffectName>Filter\|<EffectName>" silvestre-core/src/lib.rs
# Expected: no output.
```

If checks 1 or 2 fail: fix the pre-existing breakage before continuing.
If checks 3 or 4 hit: report the collision to the user and stop.

---

## Module Placement Guide

| `module-type` | Source directory | Re-export in |
|---|---|---|
| `effect` | `silvestre-core/src/effects/<snake_name>.rs` | `silvestre-core/src/effects/mod.rs` |
| `filter` | `silvestre-core/src/filters/<snake_name>.rs` | `silvestre-core/src/filters/mod.rs` |
| `transform` | `silvestre-core/src/transform/<snake_name>.rs` | `silvestre-core/src/transform/mod.rs` |

**Choosing the right module:**

- **`effect`** — per-pixel color transformation with no spatial neighborhood (examples: `Brightness`, `Sepia`, `Invert`). Use this when output pixel (x,y) depends only on input pixel (x,y).
- **`filter`** — spatial convolution or neighborhood-based operation (examples: `Gaussian`, `Sobel`, `Median`). Use when output pixel depends on a region of neighbors.
- **`transform`** — geometric operation changing image dimensions or layout (examples: `Rotate`, `Crop`, `Resize`). Use when pixel positions are remapped.

---

## Step 1 — Read the Reference Implementations

Read these files to internalize the exact project patterns before writing anything:

- `silvestre-core/src/effects/sepia.rs` — canonical per-pixel effect template
- `silvestre-core/src/effects/brightness.rs` — SIMD-accelerated effect template  
- `silvestre-core/src/simd/scalar.rs` — scalar fallback kernel pattern
- `silvestre-core/src/simd/mod.rs` — SIMD dispatch pattern
- `silvestre-core/src/filters/mod.rs` — `Filter` trait definition

---

## Step 2 — Implement the Source File

Create `silvestre-core/src/<module>/<snake_name>.rs` following this structure:

```rust
//! <One-line description of the effect>.
//!
//! <Longer explanation of the algorithm, formula, or reference standard used.>

use crate::filters::Filter;
use crate::{ColorSpace, Result, SilvestreImage};

/// <EffectName> filter.
///
/// <Detailed doc comment with the mathematical formula in a ```text block if relevant.>
///
/// # Examples
///
/// ```
/// use silvestre_core::<module>::<snake_name>::<EffectName>Filter;
/// use silvestre_core::filters::Filter;
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// // <Minimal compiling example demonstrating the key behavior.>
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct <EffectName>Filter {
    // fields for parameters (use i32/f32/u8 as appropriate; avoid usize in public API)
}

impl <EffectName>Filter {
    /// Create a new filter with the given parameters.
    #[must_use]
    pub fn new(/* params */) -> Self {
        Self { /* fields */ }
    }
    // Accessor methods (e.g. `pub fn param(&self) -> T`)
}

/// Apply <effect_name> to `image`.
///
/// Extracted as a free function so it can be called from tests and benchmarks
/// without constructing the filter struct.
pub fn <snake_name>(image: &SilvestreImage) -> Result<SilvestreImage> {
    let cs = image.color_space();
    let channels = cs.channels();
    let src = image.pixels();
    let pixel_count = (image.width() as usize) * (image.height() as usize);
    let mut dst = src.to_vec();

    for i in 0..pixel_count {
        let offset = i * channels;

        let (r, g, b) = match cs {
            ColorSpace::Grayscale => {
                let v = f32::from(src[offset]);
                (v, v, v)
            }
            ColorSpace::Rgb | ColorSpace::Rgba => {
                let r = f32::from(src[offset]);
                let g = f32::from(src[offset + 1]);
                let b = f32::from(src[offset + 2]);
                (r, g, b)
            }
        };

        // --- transform (r, g, b) → (r_out, g_out, b_out) here ---

        match cs {
            ColorSpace::Grayscale => {
                dst[offset] = /* lum */;
            }
            ColorSpace::Rgb => {
                dst[offset] = /* r_out */;
                dst[offset + 1] = /* g_out */;
                dst[offset + 2] = /* b_out */;
            }
            ColorSpace::Rgba => {
                dst[offset] = /* r_out */;
                dst[offset + 1] = /* g_out */;
                dst[offset + 2] = /* b_out */;
                // dst[offset + 3] — alpha preserved from src.to_vec()
            }
        }
    }

    SilvestreImage::new(dst, image.width(), image.height(), cs)
}

impl Filter for <EffectName>Filter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        <snake_name>(image)
    }
}
```

**Transforms that change dimensions:** the template above preserves the input size, which fits per-pixel effects and same-size transforms (`Mirror`). For dimension-changing transforms (`Crop`, `Resize`, 90° `Rotate`), compute the output `(out_w, out_h)` first, allocate `vec![0u8; out_w * out_h * channels]`, remap pixels from source coordinates, and return `SilvestreImage::new(dst, out_w, out_h, cs)`. Validate parameters (e.g. crop bounds, zero target size) and return an error rather than panicking. See `silvestre-core/src/transform/crop.rs` for the pattern.

**SIMD fast-path (for hot per-pixel paths only):**

If the effect is a simple per-channel arithmetic operation (saturating add/sub/mul, bitwise NOT, table lookup), add SIMD kernels following the pattern in `silvestre-core/src/simd/`:

1. Add a scalar reference kernel to `scalar.rs`.
2. Add x86_64 (AVX2 + SSE2 fallback), AArch64 (NEON), and WASM SIMD128 variants to their respective files.
3. Add a dispatch function in `simd/mod.rs` with the same `#[cfg]` gate structure as existing dispatchers.
4. Call the dispatcher from `<snake_name>()` instead of the inline scalar loop.

Skip SIMD for effects that cannot vectorize cleanly (e.g., HSL conversion, per-pixel coordinate math).

---

## Step 3 — Wire Up Module Exports

Open the appropriate `mod.rs` and add:

```rust
pub mod <snake_name>;
pub use <snake_name>::<EffectName>Filter;
```

Keep entries **alphabetically sorted** within their group (existing entries are sorted).

---

## Step 4 — Write the Test Suite

Every effect **must** have `#[cfg(test)]` tests in the same file. Required cases:

| Category | Tests to include |
|---|---|
| **Identity / neutral** | *If the operation has a neutral parameter:* zero-delta or identity parameter → output equals input. Skip for parameterless operations (e.g. `Invert`). |
| **Known pixel values** | Hand-compute expected output for 1–2 specific pixels and assert exactly |
| **Clamping** | Values that would overflow/underflow stay within `0..=255` |
| **Color space coverage** | One test each for `Grayscale`, `Rgb`, `Rgba` |
| **Alpha preservation** | For `Rgba`: alpha channel byte is unchanged |
| **Output dimensions** | Effects/filters: width, height, color space remain unchanged unless the effect changes color space. Dimension-changing transforms: assert the computed output width/height for representative inputs, plus an error test for invalid parameters. |
| **Empty image** | `0×0` image returns `Ok` with empty pixels |
| **Filter trait delegation** | `<EffectName>Filter.apply(img)` produces identical output to the free function |
| **Trait object** | `Box<dyn Filter>::apply(img)` compiles and produces the correct result |
| **Parameter accessors** | *If the struct exposes accessors:* `filter.param()` returns the value passed to `new()`. Skip for parameterless operations. |
| **Multi-pixel** | At least one test with a 2+ pixel image to catch off-by-one errors in the pixel loop |

Use a local helper `fn img(pixels, w, h, cs) -> SilvestreImage` that calls `SilvestreImage::new(...).unwrap()` to reduce boilerplate.

---

## Step 5 — Post-flight Verification

**Run all of these after writing the implementation. All must pass before declaring the task complete.**

```bash
# Format
cargo fmt --all

# Zero Clippy warnings
cargo clippy --workspace --all-targets -- -D warnings

# All tests (including new ones)
cargo test --workspace

# Confirm new tests specifically run and pass
cargo test -p silvestre-core <snake_name>

# Confirm docs compile with zero warnings (catches broken intra-doc links)
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

**If any step fails:** fix it before proceeding. Do not skip `--D warnings` or suppress Clippy lints without a documented reason.

---

## Step 6 — Update the Architecture Document

Open `docs/architecture/overview.md` and add the new effect name to the appropriate section in the topology diagram (Effects, Filters, or Transform box). Keep entries sorted.

---

## Completion Checklist

Before reporting the task as done, confirm every item:

- [ ] Pre-flight checks passed (clean build and tests before any changes)
- [ ] Source file created in the correct module directory
- [ ] Module `mod.rs` updated with `pub mod` and `pub use` (alphabetically sorted)
- [ ] Free function and `Filter` impl both present
- [ ] Alpha channel preserved for `Rgba` images
- [ ] `Grayscale` color space handled correctly (single channel)
- [ ] All test categories from Step 4 are covered
- [ ] `cargo fmt --all` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` passes
- [ ] Architecture doc updated
