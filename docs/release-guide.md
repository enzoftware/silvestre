# Multi-Platform Release Guide & Tag Naming Conventions

This guide establishes the versioning, tagging, and automated release strategies for all crates and packages in the Silvestre monorepo.

---

## 1. Tag Naming Conventions

Because Silvestre produces multiple distinct deliverables across different package registries (crates.io, pub.dev, npm, GitHub releases), each platform uses an **isolated tag namespace**:

| Platform / Deliverable | Tag Pattern | Example Tag | Target Registry / Artifact |
|---|---|---|---|
| **Flutter Plugin** | `flutter-v<major>.<minor>.<patch>` | `flutter-v0.1.0` | [pub.dev](https://pub.dev/packages/silvestre_flutter) |
| **Rust Core** | `core-v<major>.<minor>.<patch>` | `core-v0.1.0` | [crates.io](https://crates.io/crates/silvestre-core) |
| **CLI Binary** | `cli-v<major>.<minor>.<patch>` | `cli-v0.1.0` | GitHub Release Binaries (Linux, macOS, Windows) |
| **WebAssembly** | `wasm-v<major>.<minor>.<patch>` | `wasm-v0.1.0` | npm package (`@silvestre/wasm`) |
| **C FFI Layer** | `ffi-v<major>.<minor>.<patch>` | `ffi-v0.1.0` | GitHub Release C Header & Shared Libs |

---

## 2. Flutter Release Process (`silvestre_flutter`)

### Step-by-Step Guide:

1. **Update Version:**
   In `silvestre_flutter/pubspec.yaml`, increment the version following [Semantic Versioning](https://semver.org/):
   ```yaml
   version: 0.1.0
   ```

2. **Update Changelog:**
   In `silvestre_flutter/CHANGELOG.md`, create a section for the new version:
   ```markdown
   ## 0.1.0

   ### Added
   - Description of new features...
   ```

3. **Verify Locally:**
   ```bash
   cd silvestre_flutter
   flutter analyze
   flutter test example/test
   dart pub publish --dry-run
   ```

4. **Commit & Tag:**
   ```bash
   git add silvestre_flutter/pubspec.yaml silvestre_flutter/CHANGELOG.md
   git commit -m "chore(flutter): bump version to 0.1.0"
   git tag flutter-v0.1.0
   git push origin main --tags
   ```

5. **Automated CI/CD Execution:**
   - GitHub Actions (`.github/workflows/release-flutter.yml`) detects the `flutter-v*` tag.
   - Runs full analyzer and test suites.
   - Executes `dart pub publish --force` using automated credentials / OIDC.
   - Creates a GitHub Release with auto-generated release notes and attached changelog.

---

## 3. Automation Matrix & Best Practices

1. **Independent Release Lifecycles:**
   - Changes to the Flutter example app or Flutter-specific Dart wrappers do not force version bumps in `silvestre-core`.
   - Core algorithm improvements trigger a `core-v*` release, which downstream packages can upgrade to when ready.

2. **Dry-Run Protections:**
   - All release workflows support manual triggers via `workflow_dispatch` with a `dry_run: true` default to preview releases safely before publishing.

3. **Release Notes Generation:**
   - Release notes are automatically categorized via Conventional Commits (`feat`, `fix`, `docs`, `perf`) using GitHub's release notes generator.
