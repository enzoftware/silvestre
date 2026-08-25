# Rust Core crates.io Publication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Configure `silvestre-core` with complete metadata, add dedicated documentation, and establish an automated crates.io release pipeline on `core-v*` tags.

**Architecture:** Enrich `silvestre-core/Cargo.toml`, provide `silvestre-core/README.md`, document release commands in `docs/release-guide.md`, and configure `.github/workflows/release-core.yml`.

**Tech Stack:** Rust (`cargo`), crates.io, GitHub Actions, Markdown

**Spec:** `docs/superpowers/specs/2026-08-25-core-crates-io-publication-design.md`

## Global Constraints

- Must pass `cargo publish -p silvestre-core --dry-run` with 0 errors.
- Must preserve 100% test coverage with zero clippy warnings.

---

### Task 1: Crate Metadata & Documentation
**Files:**
- Modify: `silvestre-core/Cargo.toml`
- Create: `silvestre-core/README.md`
- Modify: `README.md`

- [x] **Step 1: Update `silvestre-core/Cargo.toml` with description, homepage, docs, keywords, categories, and readme**
- [x] **Step 2: Create `silvestre-core/README.md` with installation and API examples**
- [x] **Step 3: Update root `README.md` with `cargo add silvestre-core`**

---

### Task 2: Release Guide & Automation Workflow
**Files:**
- Modify: `docs/release-guide.md`
- Create: `.github/workflows/release-core.yml`

- [x] **Step 1: Add Rust Core release process to `docs/release-guide.md`**
- [x] **Step 2: Create `.github/workflows/release-core.yml` for automated crates.io releases**

---

### Task 3: Verification
- [x] **Step 1: Run `cargo publish -p silvestre-core --dry-run`**
  - Expected: PASS
- [x] **Step 2: Run `cargo test -p silvestre-core` and `cargo test -p silvestre-core --doc`**
  - Expected: PASS
