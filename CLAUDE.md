# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build                # dev build
cargo build --release      # release build
cargo test                 # run all tests
cargo test detection       # run tests in detection module only
cargo install --path .     # install as `tsm` to ~/.cargo/bin
./install.sh               # build + install to ~/.local/bin + add tmux keybinding
```

## Architecture

Rust + ratatui TUI app for managing tmux sessions with Claude Code status detection. Binary name: `tsm`.

### Event Loop (`main.rs`)
100ms polling loop: `app.tick()` refreshes sessions, then renders UI, then handles input.

### State Machine (`app/mode.rs` → `input.rs`)
`Mode` enum drives all UI behavior. Each mode has a dedicated key handler in `input.rs`:
- `Normal` → session list navigation (j/k/↑↓), actions (n/K/r/Enter)
- `ActionMenu` → expanded inline menu for selected session
- `NewSession` → 3-field dialog (StartWith/Name/Path) with path completion
- `Filter` / `Rename` / `ConfirmAction` / `Help`

### Core Modules
- **`app/mod.rs`** — `App` struct holds all state. Navigation uses `visual_order()` from `grouped_sessions()` so ↑↓ follows the grouped display order, not creation order.
- **`tmux.rs`** — All tmux CLI interaction. Parses `tmux list-sessions`/`list-panes` output. Detects Claude Code by checking if pane command is "claude".
- **`detection.rs`** — `detect_status(content)` analyzes captured pane text to determine Claude Code state (Idle/Working/WaitingInput/Unknown) by pattern matching on UI elements (input field border `─` above `❯`, "to interrupt", "Enter to select").
- **`session.rs`** — `Session`, `Pane`, `ClaudeCodeStatus` data types.
- **`completion.rs`** — `complete_path()` returns suggestions + ghost text for the path input field.

### UI Rendering (`ui/`)
- `ui/mod.rs` — Main layout: header, session list (grouped by directory), preview pane (ANSI-preserved), status bar, footer
- `ui/dialogs.rs` — Modal dialogs rendered as centered overlays with `Clear` + `Paragraph`
- `ui/help.rs` — Help modal + message overlays

### Selection & Scroll
`self.selected` is an index into `filtered_sessions()` (flat list). `grouped_sessions()` reorders by directory. `visual_order()` maps between the two so navigation follows visual order. `ScrollState` implements center-locked scrolling.
