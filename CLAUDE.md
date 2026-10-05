# Claude Code Guidelines for Silvestre

See [AGENTS.md](file:///Users/enzoftware/Projects/silvestre/AGENTS.md) for full project architecture, code conventions, and Superpowers workflow rules.

## Quick Commands

- **Build Workspace:** `cargo build --workspace`
- **Run Tests:** `cargo test --workspace`
- **Run Core Tests:** `cargo test -p silvestre-core`
- **Check Lints:** `cargo clippy --workspace --all-targets -- -D warnings`
- **Format Code:** `cargo fmt --all`
- **Launch Interactive TUI:** `cargo run -p silvestre-cli --`

## Superpowers Structure
- **Design Specs:** `docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md`
- **Implementation Plans:** `docs/superpowers/plans/YYYY-MM-DD-<topic>.md`
- **Architecture Overview:** `docs/architecture/overview.md`
- **Roadmap:** `docs/roadmap.md`

## Agent skills

### Issue tracker

Issues are tracked in GitHub Issues on `enzoftware/silvestre` (via `gh`). See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one root `GLOSSARY.md` plus `docs/adr/`. See `docs/agents/domain.md`.
