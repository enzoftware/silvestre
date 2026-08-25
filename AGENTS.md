# Agent Guidance & Contributor Standards for Silvestre

Welcome to Silvestre! This file provides comprehensive instructions for AI agents and human contributors working in this repository.

---

## 1. Project Topology & Architecture

Silvestre is organized as a multi-target workspace centered around a pure Rust core:

```text
silvestre/
├── silvestre-core/      # Pure Rust image processing engine (no platform dependencies)
├── silvestre-cli/       # Interactive Ratatui TUI & headless CLI binary
├── silvestre-ffi/       # Stable C ABI layer with cbindgen header generation
├── silvestre-wasm/      # WebAssembly bindings & browser demo application
├── silvestre_flutter/   # Flutter plugin (flutter_rust_bridge v2) & mobile app
└── docs/                # Architecture docs, roadmap, and Superpowers specs/plans
```

---

## 2. Superpowers Development Workflow

All new features, structural changes, and significant refactorings must follow the **Superpowers** workflow:

1. **Brainstorming & Design:**
   - Use the `superpowers:brainstorming` skill to explore ideas and clarify requirements.
   - Save design specifications to `docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md`.
2. **Implementation Planning:**
   - Use `superpowers:writing-plans` to break designs into bite-sized, test-driven steps.
   - Save plans to `docs/superpowers/plans/YYYY-MM-DD-<topic>.md`.
3. **Execution & TDD:**
   - Execute task-by-task using `superpowers:subagent-driven-development` or `superpowers:executing-plans`.
   - Write failing tests first before implementing functionality.
4. **Review & Finishing:**
   - Request review using `superpowers:requesting-code-review`.
   - Conclude branches using `superpowers:finishing-a-development-branch`.

---

## 3. Code Conventions & Standards

### Rust (`silvestre-core`, `silvestre-cli`, `silvestre-ffi`, `silvestre-wasm`)
- **Formatting:** Format all code with `cargo fmt --all`.
- **Linting:** Zero Clippy warnings allowed (`cargo clippy --workspace --all-targets -- -D warnings`).
- **Safety:** Avoid `unsafe` blocks in `silvestre-core`. In FFI layers, validate all incoming raw pointers for null/bounds.
- **Error Handling:** Use `thiserror` for library error enums and propagate errors with `Result<T, SilvestreError>`.

### Dart / Flutter (`silvestre_flutter`)
- Follow official Dart style and `flutter_lints`.
- Use the Flutter BLoC pattern for asynchronous state management.
- Keep heavy compute operations off the main UI isolate.

---

## 4. Commit & Pull Request Standards

### Commit Messages
Follow Conventional Commits format (`feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`):
- `feat(core): add bilateral filter`
- `fix(transform): handle zero-dimension crops gracefully`
- `docs(specs): add design spec for GPU compute`

### Pull Requests
Pull request descriptions should include:
- `## Summary` — Bulleted explanation of changes and rationale.
- `## Test plan` — Verified checklist of commands run and results.
- `Closes #<issue>` — Reference to relevant issues.

---

## 5. Release Conventions & Tagging

Each platform deliverable has an isolated tag namespace (see [docs/release-guide.md](docs/release-guide.md) for full details):
- **Flutter:** `flutter-v<version>` (publishes to pub.dev)
- **Rust Core:** `core-v<version>` (publishes to crates.io)
- **CLI Binary:** `cli-v<version>` (GitHub Release binaries)
- **WebAssembly:** `wasm-v<version>` (publishes to npm)
- **C FFI:** `ffi-v<version>` (GitHub Release C headers & static libs)

---

## 6. Verification Commands

```bash
# Test entire Rust workspace
cargo test --workspace

# Test specific core crate
cargo test -p silvestre-core

# Check formatting and lints
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# Build documentation
cargo doc --workspace --no-deps
```
