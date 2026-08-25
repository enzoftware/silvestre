# Documentation Restructuring & Superpowers Architecture Specification

**Date:** 2026-08-25  
**Topic:** Superpowers Documentation Migration & Retroactive Specs  
**Status:** Approved  

---

## 1. Overview

This document specifies the restructuring of the Silvestre project's documentation to conform to the **Superpowers** workflow standards. It defines the migration of root-level planning materials, retroactive creation of design specifications and implementation plans for all merged pull requests (#41–#65), creation of unified agent guidance files (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`), and centralization of architecture and roadmap documentation.

---

## 2. Directory Layout & Architecture

The target repository structure for documentation and agent guidance is defined as:

```text
silvestre/
├── AGENTS.md                            # Universal guidelines for AI agents & contributors
├── CLAUDE.md                            # Claude Code project entry & tool bindings
├── GEMINI.md                            # Gemini CLI project entry & tool bindings
├── README.md                            # Public overview with updated documentation index
├── Cargo.toml                           # Cargo workspace configuration
├── docs/
│   ├── architecture/
│   │   └── overview.md                  # Comprehensive architectural overview & crate interactions
│   ├── roadmap.md                       # Milestones, completed phases, and future roadmap
│   └── superpowers/
│       ├── specs/                       # Design specifications (YYYY-MM-DD-<topic>-design.md)
│       │   ├── 2026-04-05-core-architecture-design.md
│       │   ├── 2026-04-10-filters-effects-transforms-design.md
│       │   ├── 2026-04-28-rotate-filter-design.md
│       │   ├── 2026-04-29-cli-interactive-tui-design.md
│       │   ├── 2026-04-29-c-ffi-layer-design.md
│       │   ├── 2026-04-30-wasm-web-demo-design.md
│       │   ├── 2026-04-30-flutter-plugin-and-app-design.md
│       │   ├── 2026-05-01-ci-and-release-design.md
│       │   └── 2026-08-25-documentation-restructuring-design.md
│       └── plans/                       # Implementation plans (YYYY-MM-DD-<topic>.md)
│           ├── 2026-04-05-core-foundation.md
│           ├── 2026-04-28-filter-improvements.md
│           ├── 2026-04-29-cli-tui.md
│           ├── 2026-04-29-c-ffi.md
│           ├── 2026-04-30-wasm.md
│           ├── 2026-04-30-flutter.md
│           ├── 2026-05-01-ci-automation.md
│           └── 2026-08-25-documentation-restructuring.md
```

---

## 3. Detailed File Specifications

### 3.1 Architecture Overview (`docs/architecture/overview.md`)
- **Origin:** Extracted and expanded from root `PLAN.md`.
- **Contents:**
  - Full ASCII architectural diagram showing multi-target platform layers (CLI, Flutter, WebAssembly, C ABI / FFI) over pure-Rust `silvestre-core`.
  - Core data structures: `SilvestreImage`, `ColorSpace` (Rgb, Rgba, Grayscale), `Filter` trait definition.
  - Convolution engine: `Kernel` and `SeparableKernel` mechanisms.
  - Memory models: Rust buffer ownership, FFI opaque handles and pointers, WebAssembly linear memory sharing via typed arrays.
  - Inter-crate communication and dependency relationships.

### 3.2 Roadmap & Milestones (`docs/roadmap.md`)
- **Origin:** Extracted and expanded from root `PLAN.md`.
- **Contents:**
  - Record of completed phases (Phase 1 through Phase 6) mapping directly to merged PRs #41–#65.
  - Current status of the workspace (Core, FFI, CLI, WASM, Flutter, CI).
  - Future milestones & roadmap (GPU acceleration via `wgpu`, streaming tile-based processing, WASM custom filter plugin system, video frame processing, Python bindings via PyO3).

### 3.3 Retroactive Design Specs (`docs/superpowers/specs/`)

1. **`2026-04-05-core-architecture-design.md`**
   - **PRs:** #41, #42, #43, #44
   - **Scope:** Workspace initialization, `SilvestreImage` byte buffer design, image format encodings/decodings (PNG, JPEG, BMP), `Filter` trait, and separable convolution infrastructure.

2. **`2026-04-10-filters-effects-transforms-design.md`**
   - **PRs:** #45, #46, #47, #48, #49, #50, #51, #52, #53, #54, #56, #58
   - **Scope:** Spatial filters (Median, Gaussian, Sobel, Canny), color effects (Grayscale, Sepia, Invert, Brightness, Contrast), geometric transforms (Resize with nearest & bilinear, Mirror horizontal/vertical/both, Crop), histogram analysis, and test suites.

3. **`2026-04-29-cli-interactive-tui-design.md`**
   - **PRs:** #57, #62
   - **Scope:** Interactive terminal UI via `ratatui` and `crossterm`, half-block/ASCII image renderers, real-time parameter tweaking modals, and multi-filter pipeline builder.

4. **`2026-04-29-c-ffi-layer-design.md`**
   - **PRs:** #59
   - **Scope:** C-compatible ABI exports in `silvestre-ffi`, opaque `SilvestreImageHandle`, C-string and error handling via `silvestre_last_error`, memory deallocation routines, and `cbindgen.toml` header generation (`silvestre.h`).

5. **`2026-04-30-wasm-web-demo-design.md`**
   - **PRs:** #60
   - **Scope:** `wasm-bindgen` interface for browser execution, direct `Uint8ClampedArray` / `ImageData` byte manipulations, web application demo UI (`index.html`, `index.js`), and `wasm-pack` compilation pipeline.

6. **`2026-04-30-flutter-plugin-and-app-design.md`**
   - **PRs:** #61, #63
   - **Scope:** `flutter_rust_bridge` v2 integration, Dart FFI bindings, BLoC architecture for state management, Flutter example app with camera feed, before/after split slider widget, and image gallery persistence.

7. **`2026-05-01-ci-and-release-design.md`**
   - **PRs:** #64, #65
   - **Scope:** GitHub Actions CI matrix across Linux/macOS/Windows, `cargo test --workspace`, `wasm-pack test`, `flutter test`, Clippy, rustfmt, cargo docs, and Criterion performance benchmarking.

### 3.4 Retroactive Implementation Plans (`docs/superpowers/plans/`)

Standard Superpowers bite-sized task plans with explicit test commands:
- `2026-04-05-core-foundation.md` (Phase 1)
- `2026-04-29-cli-tui.md` (Phase 2)
- `2026-04-29-c-ffi.md` (Phase 3)
- `2026-04-30-wasm.md` (Phase 4)
- `2026-04-30-flutter.md` (Phase 5)
- `2026-05-01-ci-automation.md` (Phase 6)
- `2026-08-25-documentation-restructuring.md` (Current restructuring implementation plan)

### 3.5 Agent Guidelines & Project Entry Points

1. **`AGENTS.md`**: Universal instructions across all agent frameworks detailing:
   - Repository architecture and crate roles.
   - Code conventions (Rust idioms, error handling with `thiserror`, Dart BLoC style, TypeScript).
   - Commit standards (`conventional-commit`).
   - Superpowers workflow (Brainstorming → Spec → Plan → Subagent Execution → Review → Finishing Branch).
   - Workspace build and test commands.
2. **`CLAUDE.md` & `GEMINI.md`**: Tool command mappings, quick commands, and references to `AGENTS.md` and `docs/superpowers/`.

### 3.6 Root Migration & `README.md`
- Delete root `PLAN.md` once all materials are integrated into `docs/architecture/overview.md` and `docs/roadmap.md`.
- Update `README.md` to reflect the clean project structure and link to the documentation index.

---

## 4. Verification & Validation Criteria

1. **Document Integrity:** All links between markdown documents (`docs/`, `AGENTS.md`, `README.md`, specs, plans) resolve correctly.
2. **Codebase Cleanliness:** Root directory contains only root config files and entry points; all planning and architecture docs reside under `docs/`.
3. **Workspace Health:** `cargo check --workspace` and `cargo test -p silvestre-core` execute without error.
4. **Git State:** Clean commit history on feature branch ready for PR creation.
