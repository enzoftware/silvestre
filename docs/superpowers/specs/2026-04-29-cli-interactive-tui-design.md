# CLI Tool & Interactive TUI Design Specification

**Date:** 2026-04-29  
**Topic:** Command-Line Interface, Ratatui Terminal UI, and Filter Pipelines  
**Pull Requests:** #57, #62  
**Status:** Implemented  

---

## 1. Overview

This specification details the architecture of `silvestre-cli`, providing both headless CLI automation and an interactive, keyboard-driven Terminal User Interface (TUI) powered by `ratatui` and `crossterm`.

---

## 2. Command-Line Interface (`silvestre-cli/src/commands/`)

Command parsing is powered by `clap`:

### 2.1 Subcommands
1. **`apply`**: Non-interactive command to apply one or more filters and save the output.
   ```bash
   silvestre apply --input input.jpg --output output.png --filter gaussian:sigma=2.0 --filter grayscale
   ```
2. **`info`**: Inspect image dimensions, color space, bit depth, and histogram statistics.
3. **`list`**: Display all available filters, accepted parameters, and default values.

---

## 3. Interactive Terminal User Interface (`silvestre-cli/src/ui.rs`)

### 3.1 Terminal Image Rendering
- Employs Unicode half-block characters (`▀` and `▄`) to display RGB true-color pixels directly in ANSI-compatible terminals, mapping top and bottom subpixels to foreground and background ANSI escape codes.
- Downsamples large images proportionally to fit within terminal viewport dimensions.

### 3.2 Layout & Component Architecture
The TUI is split into 4 primary panels:
1. **Left Sidebar:** Filter selection list and active pipeline stages.
2. **Main Canvas:** Live rendered image preview reflecting currently applied filter chain.
3. **Bottom Bar:** Keyboard shortcut guide and status messages.
4. **Parameter Modal / Popup:** Interactive numerical sliders and options for fine-tuning filter arguments.

### 3.3 State Machine & Event Loop (`app.rs`, `handlers.rs`)
- **Events:** Key inputs (Up/Down for selection, Enter to apply, Tab to switch panels, Space to toggle stage).
- **Pipeline Evaluation:** Filters are re-evaluated asynchronously or debounced upon parameter adjustments.

---

## 4. Verification

- Headless argument parsing and filter execution tests.
- Terminal rendering resolution downsampling tests.
- Multi-filter chained pipeline correctness tests.
