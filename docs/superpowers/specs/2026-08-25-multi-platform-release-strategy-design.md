# Multi-Platform Release Strategy & Automated Documentation Specification

**Date:** 2026-08-25  
**Topic:** Release Namespaces, Automation Pipelines, and Documentation Sync  
**Status:** Approved  

---

## 1. Overview

This design specification establishes the multi-platform release strategy and automated documentation synchronization pipeline for the Silvestre repository. It guarantees that every target crate/package has a clearly isolated versioning lifecycle and that project documentation is automatically verified without manual intervention.

---

## 2. Release Namespaces & Tag Conventions

To manage independent release schedules within a unified monorepo:

1. **`flutter-v*`**: Triggers pub.dev validation and automated publication for `silvestre_flutter`.
2. **`core-v*`**: Triggers crates.io publication for `silvestre-core`.
3. **`cli-v*`**: Triggers multi-platform binary compilation and GitHub release asset upload for `silvestre-cli`.
4. **`wasm-v*`**: Triggers npm publishing for `@silvestre/wasm`.
5. **`ffi-v*`**: Triggers C header (`silvestre.h`) and shared library artifact builds.

---

## 3. Automated Documentation Architecture

To maintain documentation freshness without manual intervention:
1. **GitHub Actions Doc-Test Gate (`.github/workflows/docs.yml`):**
   - Automatically runs `cargo test --workspace --doc` on every PR that touches source code or documentation.
   - Executes `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"` to detect broken links or outdated references.
2. **Release Notes Synthesis:**
   - GitHub Releases automatically aggregate commits between version tags and group them into `Features`, `Bug Fixes`, and `Documentation`.
3. **Continuous Plan Tracking:**
   - Superpowers plans in `docs/superpowers/plans/` record task deliverables with explicit checkboxes, serving as an immutable audit trail.
