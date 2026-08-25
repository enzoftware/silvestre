# Rust Core crates.io Publication Specification

**Date:** 2026-08-25  
**Topic:** crates.io Distribution, Crate Metadata, and Release Automation  
**Issue:** #36  
**Status:** Implemented  

---

## 1. Overview

This design specification details the preparation, packaging, documentation, and continuous release automation for publishing `silvestre-core` to [crates.io](https://crates.io/crates/silvestre-core) for the Rust ecosystem.

---

## 2. Crate Configuration & Metadata (`silvestre-core/Cargo.toml`)

- **Crate Name:** `silvestre-core`
- **Description:** `"Pure Rust image processing library providing composable filters, effects, transformations, and histogram analysis."`
- **License:** `MIT`
- **Homepage:** `https://github.com/enzoftware/silvestre`
- **Documentation:** `https://docs.rs/silvestre-core`
- **Keywords:** `["image-processing", "filters", "canny", "sobel", "computer-vision"]`
- **Categories:** `["multimedia::images", "algorithms"]`
- **Readme:** `README.md`

---

## 3. Documentation (`silvestre-core/README.md`)

- Detailed Rust code snippets demonstrating image loading, spatial convolution filters, color adjustments, geometric transforms, histogram statistical analysis, and file saving.
- Complete documentation of the `Filter` trait, thread-safety guarantees (`Send + Sync`), and zero platform dependency design.

---

## 4. Automated Release Workflow (`.github/workflows/release-core.yml`)

- **Trigger:** Tag push matching `core-v*` or manual `workflow_dispatch`.
- **Pipeline:**
  1. Setup Rust toolchain with cache.
  2. Execute unit and integration tests: `cargo test -p silvestre-core`.
  3. Execute doc-tests: `cargo test -p silvestre-core --doc`.
  4. Run package validation: `cargo publish -p silvestre-core --dry-run`.
  5. Publish to crates.io with `cargo publish -p silvestre-core --token ${{ secrets.CRATES_IO_TOKEN }}`.
  6. Create a tagged GitHub Release with automated changelog notes.

---

## 5. Verification

- `cargo publish -p silvestre-core --dry-run`: Successful packaging (32 files, 288 KiB, zero errors).
- Full test pass: `cargo test -p silvestre-core`.
- Documentation generation: `cargo doc -p silvestre-core --no-deps`.
