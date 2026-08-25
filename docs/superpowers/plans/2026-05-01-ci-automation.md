# CI Automation, Quality Gates & Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish GitHub Actions continuous integration pipelines across Ubuntu, macOS, and Windows for all workspace targets, along with API docs and linting gates.

**Architecture:** Configure workflow jobs for Rust workspace tests, clippy, rustfmt, WebAssembly tests, and Flutter analysis.

**Tech Stack:** GitHub Actions, Rust (`cargo`), `wasm-pack`, Flutter CLI

**Spec:** `docs/superpowers/specs/2026-05-01-ci-and-release-design.md`

## Global Constraints

- CI must pass cleanly on all 3 target OS platforms before pull requests can be merged.

---

### Task 1: GitHub Actions CI Configuration
**Files:**
- Create: `.github/workflows/ci.yml`

- [x] **Step 1: Set up multi-OS build matrix**
- [x] **Step 2: Add test, clippy, and rustfmt steps**
- [x] **Step 3: Add WASM and Flutter verification steps**

---

### Task 2: Documentation & README Guides
**Files:**
- Create: `silvestre-wasm/README.md`
- Create: `silvestre_flutter/README.md`
- Create: `silvestre-cli/README.md`
- Create: `silvestre-ffi/README.md`

- [x] **Step 1: Write platform-specific README files**
- [x] **Step 2: Verify `cargo doc` compilation without broken links**
