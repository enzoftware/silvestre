# Multi-Platform Release & Docs Automation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the multi-platform release strategy with isolated tag namespaces, provide a comprehensive release guide, fix clippy/compiler errors, and automate documentation validation.

**Architecture:** Create `.github/workflows/docs.yml`, write `docs/release-guide.md`, update `AGENTS.md`, and resolve slice chunking clippy lints across `silvestre-core` and `silvestre-wasm`.

**Tech Stack:** Rust (`cargo`), Flutter, GitHub Actions, Markdown

**Spec:** `docs/superpowers/specs/2026-08-25-multi-platform-release-strategy-design.md`

## Global Constraints

- Never break backward compatibility for existing release tags.
- Ensure all doc-tests pass with `cargo test --workspace --doc`.

---

### Task 1: Fix Clippy Chunking Errors
**Files:**
- Modify: `silvestre-core/src/analysis/histogram.rs`
- Modify: `silvestre-wasm/src/lib.rs`

- [x] **Step 1: Replace `chunks_exact` with `as_chunks` for constant sizes**
- [x] **Step 2: Run `cargo clippy --workspace --all-targets -- -D warnings`**
  - Expected: PASS

---

### Task 2: Release Guide & Tag Taxonomy
**Files:**
- Create: `docs/release-guide.md`
- Create: `.github/workflows/release-flutter.yml`

- [x] **Step 1: Document platform tag namespaces (`flutter-v*`, `core-v*`, `cli-v*`, etc.)**
- [x] **Step 2: Document step-by-step Flutter publishing process**
- [x] **Step 3: Add automated release workflow in GitHub Actions**

---

### Task 3: Automated Documentation Pipeline
**Files:**
- Create: `.github/workflows/docs.yml`

- [x] **Step 1: Create docs validation workflow for doc-tests and rustdoc**
- [x] **Step 2: Verify `cargo doc --workspace --no-deps`**
