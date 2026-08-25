# WebAssembly npm Publication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prepare and configure `silvestre-wasm` for npm publishing, add bundler documentation and type guides, and set up automated GitHub Actions release workflows.

**Architecture:** Package `silvestre-wasm` via `wasm-pack build --target bundler`, provide MIT license in the crate, and automate npm publishing on `wasm-v*` tags.

**Tech Stack:** Rust (`wasm32-unknown-unknown`), `wasm-pack`, npm, GitHub Actions, TypeScript

**Spec:** `docs/superpowers/specs/2026-08-25-wasm-npm-publication-design.md`

## Global Constraints

- Must work with modern bundlers (Webpack 5, Vite, Next.js).
- Must include complete TypeScript type definitions.
- Must pass `npm publish --dry-run` with 0 errors.

---

### Task 1: Package Configuration & Licenses
**Files:**
- Create: `silvestre-wasm/LICENSE`
- Modify: `silvestre-wasm/Cargo.toml`

- [x] **Step 1: Add MIT LICENSE to `silvestre-wasm/`**
- [x] **Step 2: Update `Cargo.toml` with description, homepage, repository, keywords, and categories**

---

### Task 2: Documentation & Bundler Guides
**Files:**
- Modify: `silvestre-wasm/README.md`
- Modify: `docs/release-guide.md`
- Modify: `README.md`

- [x] **Step 1: Add npm installation instructions and bundler configs (Vite, Webpack 5, Next.js) to `silvestre-wasm/README.md`**
- [x] **Step 2: Add WASM release guide to `docs/release-guide.md`**
- [x] **Step 3: Update root `README.md` with npm package reference**

---

### Task 3: Automated Release Workflow
**Files:**
- Create: `.github/workflows/release-wasm.yml`

- [x] **Step 1: Add GitHub Actions workflow for `wasm-v*` release tags**
- [x] **Step 2: Test `wasm-pack build --target bundler` and `npm publish --dry-run`**
  - Run: `wasm-pack build --target bundler --out-dir pkg && cd pkg && npm publish --dry-run`
  - Expected: PASS
