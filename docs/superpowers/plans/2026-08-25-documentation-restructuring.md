# Documentation Restructuring & Superpowers Migration Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate and restructure repository documentation into the Superpowers workflow structure, generate retroactive design specs and implementation plans for all merged pull requests, and configure universal agent guides (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`).

**Architecture:** Centralize all architectural docs under `docs/architecture/` and roadmaps under `docs/roadmap.md`, move historical planning from root `PLAN.md`, populate `docs/superpowers/specs/` and `docs/superpowers/plans/` for the 6 implementation milestones, and update `README.md` and agent instructions.

**Tech Stack:** Markdown, Rust (`cargo`), Git, GitHub PR

**Spec:** `docs/superpowers/specs/2026-08-25-documentation-restructuring-design.md`

## Global Constraints

- Preserve all accurate architectural diagrams, type signatures, and design rationales from the original `PLAN.md` and codebase.
- Adhere strictly to the Superpowers file naming conventions (`YYYY-MM-DD-<topic>-design.md` and `YYYY-MM-DD-<topic>.md`).
- Ensure all relative markdown links across `README.md`, `AGENTS.md`, `docs/roadmap.md`, and `docs/architecture/` are valid and reachable.
- Verify workspace builds and tests pass throughout (`cargo test -p silvestre-core`).

---

### Task 1: Create Architecture Overview and Roadmap Documents

**Files:**
- Create: `docs/architecture/overview.md`
- Create: `docs/roadmap.md`
- Delete: `PLAN.md`

- [ ] **Step 1: Write `docs/architecture/overview.md`**
  - Include multi-platform target diagram (CLI, Flutter, WebAssembly, C ABI).
  - Document `SilvestreImage`, `ColorSpace`, and `Filter` trait.
  - Document convolution infrastructure and FFI/WASM memory models.
- [ ] **Step 2: Write `docs/roadmap.md`**
  - Document completed milestones Phase 1 to Phase 6 with PR references.
  - Document upcoming features (GPU acceleration, streaming, WASM plugins, video, Python bindings).
- [ ] **Step 3: Remove root `PLAN.md`**
- [ ] **Step 4: Commit Task 1**
  - `git add docs/architecture/overview.md docs/roadmap.md PLAN.md`
  - `git commit -m "docs: create architecture overview and roadmap, remove root PLAN.md"`

---

### Task 2: Generate Retroactive Design Specs in `docs/superpowers/specs/`

**Files:**
- Create: `docs/superpowers/specs/2026-04-05-core-architecture-design.md`
- Create: `docs/superpowers/specs/2026-04-10-filters-effects-transforms-design.md`
- Create: `docs/superpowers/specs/2026-04-29-cli-interactive-tui-design.md`
- Create: `docs/superpowers/specs/2026-04-29-c-ffi-layer-design.md`
- Create: `docs/superpowers/specs/2026-04-30-wasm-web-demo-design.md`
- Create: `docs/superpowers/specs/2026-04-30-flutter-plugin-and-app-design.md`
- Create: `docs/superpowers/specs/2026-05-01-ci-and-release-design.md`

- [ ] **Step 1: Write Core Architecture Spec (`2026-04-05-core-architecture-design.md`)**
  - Cover PRs #41–#44: Workspace scaffold, `SilvestreImage`, I/O, `Filter` trait & convolution.
- [ ] **Step 2: Write Filters, Effects & Transforms Spec (`2026-04-10-filters-effects-transforms-design.md`)**
  - Cover PRs #45–#54, #56, #58: Spatial filters, edge detectors, color adjustments, resize, mirror, crop, histogram analysis.
- [ ] **Step 3: Write CLI & Interactive TUI Spec (`2026-04-29-cli-interactive-tui-design.md`)**
  - Cover PRs #57, #62: Ratatui TUI, live image preview, filter parameter modals, pipeline runner.
- [ ] **Step 4: Write C FFI Layer Spec (`2026-04-29-c-ffi-layer-design.md`)**
  - Cover PR #59: C ABI, opaque handles, `cbindgen` header generation, error strings.
- [ ] **Step 5: Write WebAssembly Spec (`2026-04-30-wasm-web-demo-design.md`)**
  - Cover PR #60: `silvestre-wasm`, `wasm-bindgen`, raw pixel buffers, web demo.
- [ ] **Step 6: Write Flutter Plugin Spec (`2026-04-30-flutter-plugin-and-app-design.md`)**
  - Cover PRs #61, #63: `silvestre_flutter`, `flutter_rust_bridge`, BLoC pattern, camera & before/after slider.
- [ ] **Step 7: Write CI & Release Spec (`2026-05-01-ci-and-release-design.md`)**
  - Cover PRs #64, #65: GitHub Actions matrix, multi-target testing, benchmarks, API docs.
- [ ] **Step 8: Commit Task 2**
  - `git add docs/superpowers/specs/`
  - `git commit -m "docs(specs): add retroactive design specs for milestones 1 through 6"`

---

### Task 3: Generate Retroactive Implementation Plans in `docs/superpowers/plans/`

**Files:**
- Create: `docs/superpowers/plans/2026-04-05-core-foundation.md`
- Create: `docs/superpowers/plans/2026-04-29-cli-tui.md`
- Create: `docs/superpowers/plans/2026-04-29-c-ffi.md`
- Create: `docs/superpowers/plans/2026-04-30-wasm.md`
- Create: `docs/superpowers/plans/2026-04-30-flutter.md`
- Create: `docs/superpowers/plans/2026-05-01-ci-automation.md`

- [ ] **Step 1: Write Core Foundation Plan (`2026-04-05-core-foundation.md`)**
  - Tasks for workspace initialization, image types, filters, transforms, and unit tests.
- [ ] **Step 2: Write CLI TUI Plan (`2026-04-29-cli-tui.md`)**
  - Tasks for clap commands, ratatui TUI screens, parameter editors, and pipeline execution.
- [ ] **Step 3: Write C FFI Plan (`2026-04-29-c-ffi.md`)**
  - Tasks for C ABI functions, pointer safety, cbindgen build script, and verification.
- [ ] **Step 4: Write WASM Plan (`2026-04-30-wasm.md`)**
  - Tasks for wasm-bindgen exports, canvas bridge, and web demo app.
- [ ] **Step 5: Write Flutter Plan (`2026-04-30-flutter.md`)**
  - Tasks for flutter_rust_bridge scaffold, Dart API, BLoC state management, and mobile UI demo.
- [ ] **Step 6: Write CI Automation Plan (`2026-05-01-ci-automation.md`)**
  - Tasks for GitHub Actions workflows, multi-platform test matrices, and doc generation.
- [ ] **Step 7: Commit Task 3**
  - `git add docs/superpowers/plans/`
  - `git commit -m "docs(plans): add retroactive implementation plans for milestones 1 through 6"`

---

### Task 4: Configure Universal Agent Guidelines & Entry Points

**Files:**
- Create: `AGENTS.md`
- Create: `CLAUDE.md`
- Create: `GEMINI.md`

- [ ] **Step 1: Write `AGENTS.md`**
  - Comprehensive guide covering repo architecture, code style, commit conventions, testing commands, and Superpowers spec/plan rules.
- [ ] **Step 2: Write `CLAUDE.md`**
  - Claude-specific shortcuts, tool mappings, and reference to `AGENTS.md`.
- [ ] **Step 3: Write `GEMINI.md`**
  - Gemini CLI-specific tool mappings, activate_skill bindings, and reference to `AGENTS.md`.
- [ ] **Step 4: Commit Task 4**
  - `git add AGENTS.md CLAUDE.md GEMINI.md`
  - `git commit -m "docs: add AGENTS.md, CLAUDE.md, and GEMINI.md project guidelines"`

---

### Task 5: Update `README.md` and Project Layout References

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Update repository structure diagram in `README.md`**
- [ ] **Step 2: Add Documentation & Superpowers Index to `README.md`**
- [ ] **Step 3: Commit Task 5**
  - `git add README.md`
  - `git commit -m "docs: update README with new documentation structure and links"`

---

### Task 6: Final Verification & Pull Request Creation

- [ ] **Step 1: Run workspace verification**
  - Run: `cargo test -p silvestre-core`
  - Expected: PASS
- [ ] **Step 2: Check git status and branch diff**
  - Run: `git status`
- [ ] **Step 3: Push branch and create Pull Request**
  - Run: `git push -u origin docs/superpowers-restructure`
  - Run: `gh pr create --title "docs: restructure repository for superpowers architecture" --body "..."`
