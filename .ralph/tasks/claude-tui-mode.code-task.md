---
status: completed
created: 2026-01-14
started: 2026-01-14
completed: 2026-01-14
---
# Code Task: Enable Claude TUI Mode (Interactive without `-p`)

## Overview

Fix the TUI to work with `claude --dangerously-skip-permissions <prompt>` (without `-p` flag). Currently, the terminal pane is not displaying Claude's native TUI correctly via ratatui. Keep the original `claude -p` with `--output-format stream-json` for non-TUI autonomous mode.

## Problem Statement

When Claude is run without the `-p` flag, it enters interactive mode with its own TUI (spinners, colored output, cursor positioning). The current Ralph TUI has issues displaying this correctly:

1. **Fixed terminal size**: `TerminalWidget` creates a parser with hardcoded size `Parser::new(24, 80, 0)` that doesn't match the PTY size
2. **No resize synchronization**: The PTY is spawned with size from `PtyConfig::from_env()`, but the `TerminalWidget` doesn't resize to match
3. **Wrong execution mode for TUI**: Claude backend always uses `-p` and `--output-format stream-json`, even for TUI mode

## Architecture Context

**Current Flow (Non-TUI/Autonomous):**
```
ralph run -p "prompt"
  → CliBackend::claude() builds: claude --dangerously-skip-permissions --verbose --output-format stream-json -p "prompt"
  → PtyExecutor::run_observe_streaming() parses NDJSON
  → Output goes to console/stream handler
```

**Desired Flow (TUI/Interactive):**
```
ralph run --tui -p "prompt"
  → CliBackend::claude_tui() builds: claude --dangerously-skip-permissions "prompt" (no -p, no stream-json)
  → PtyExecutor::run_interactive() handles raw PTY I/O
  → TUI renders Claude's native TUI via tui_term::PseudoTerminal
  → TerminalWidget resizes to match PTY dimensions
```

## Requirements

### 1. Add Claude TUI backend mode

**File:** `crates/ralph-adapters/src/cli_backend.rs`

Add a new method `claude_tui()` that builds a command without `-p` or `--output-format stream-json`:

```rust
/// Creates the Claude TUI backend for interactive mode.
///
/// Runs Claude in full interactive mode (no -p flag), allowing
/// Claude's native TUI to render. The prompt is passed as a
/// positional argument.
pub fn claude_tui() -> Self {
    Self {
        command: "claude".to_string(),
        args: vec!["--dangerously-skip-permissions".to_string()],
        prompt_mode: PromptMode::Arg,
        prompt_flag: None,  // No -p flag - prompt is positional
        output_format: OutputFormat::Text,  // Not stream-json
    }
}
```

### 2. Select backend mode based on execution context

**File:** `crates/ralph-cli/src/main.rs`

When creating the backend, choose between `claude()` and `claude_tui()` based on whether TUI is enabled:

- If `enable_tui && config.cli.backend == "claude"` → use `CliBackend::claude_tui()`
- Otherwise → use `CliBackend::from_config()` (existing behavior)

Update around line 999-1000:

```rust
// Create backend - use TUI mode for Claude when TUI is enabled
let backend = if enable_tui && config.cli.backend == "claude" {
    CliBackend::claude_tui()
} else {
    CliBackend::from_config(&config.cli)
        .map_err(|e| anyhow::Error::new(e))?
};
```

### 3. Synchronize terminal widget size with PTY

**File:** `crates/ralph-tui/src/widgets/terminal.rs`

The `TerminalWidget::new()` should accept dimensions and `resize()` should be used:

```rust
impl TerminalWidget {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(24, 80, 0),  // Default, will be resized
        }
    }

    pub fn with_size(rows: u16, cols: u16) -> Self {
        Self {
            parser: Parser::new(rows, cols, 0),
        }
    }

    /// Resizes the terminal to new dimensions.
    /// Creates a new parser to reset state for the new size.
    pub fn resize(&mut self, rows: u16, cols: u16) {
        // Only resize if dimensions actually changed
        let (current_rows, current_cols) = self.parser.screen().size();
        if current_rows != rows || current_cols != cols {
            self.parser = Parser::new(rows, cols, 0);
        }
    }
}
```

### 4. Resize terminal widget on TUI layout changes

**File:** `crates/ralph-tui/src/app.rs`

After computing the layout chunks, resize the terminal widget to match the available area:

```rust
terminal.draw(|f| {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Min(0),      // Terminal
            Constraint::Length(3),   // Footer
        ])
        .split(f.area());

    // Resize terminal widget to match available space
    let terminal_area = chunks[1];
    {
        let mut widget = self.terminal_widget.lock().unwrap();
        widget.resize(terminal_area.height, terminal_area.width);
    }

    // ... rest of rendering
})?;
```

### 5. Notify PTY of terminal resize

**File:** `crates/ralph-adapters/src/pty_handle.rs`

The `ControlCommand::Resize` already exists. Ensure it's sent when terminal size changes:

**File:** `crates/ralph-tui/src/app.rs`

Add resize tracking and control command sending:

```rust
// Track previous size to detect changes
let mut last_size: Option<(u16, u16)> = None;

// In the draw loop:
let terminal_area = chunks[1];
let new_size = (terminal_area.height, terminal_area.width);
if last_size != Some(new_size) {
    last_size = Some(new_size);
    let _ = self.control_tx.send(ControlCommand::Resize {
        rows: new_size.0,
        cols: new_size.1
    });
}
```

### 6. Handle resize in PTY executor

**File:** `crates/ralph-adapters/src/pty_executor.rs`

Ensure `ControlCommand::Resize` is handled in the `run_interactive` select loop to resize the PTY:

```rust
// In the control command handling:
PtyControl::Resize { rows, cols } => {
    // Resize the PTY
    let _ = pair.master.resize(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    });
}
```

## Files to Modify

1. `crates/ralph-adapters/src/cli_backend.rs` - Add `claude_tui()` method
2. `crates/ralph-cli/src/main.rs` - Select backend based on TUI mode
3. `crates/ralph-tui/src/widgets/terminal.rs` - Add `with_size()` and improve `resize()`
4. `crates/ralph-tui/src/app.rs` - Resize widget on layout changes, send resize commands
5. `crates/ralph-adapters/src/pty_executor.rs` - Handle resize control command

## Acceptance Criteria

- [ ] `ralph run --tui -p "hello"` with Claude backend shows Claude's native TUI correctly
- [ ] `ralph run -p "hello"` (autonomous mode) still uses `-p` with `stream-json` output
- [ ] Terminal resizes when window size changes
- [ ] Claude's spinners and colored output render correctly in TUI mode
- [ ] All existing tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)

## Test Plan

1. **Manual TUI test**:
   ```bash
   cargo build --release
   ./target/release/ralph run --tui -c ralph.claude.yml -p "Say hello"
   ```
   - Verify Claude's TUI renders correctly (no garbled output)
   - Resize the terminal window, verify content adjusts

2. **Manual autonomous test**:
   ```bash
   ./target/release/ralph run -c ralph.claude.yml -p "Say hello"
   ```
   - Verify NDJSON output is parsed correctly
   - Verify events are extracted from stream

3. **Unit tests**:
   ```bash
   cargo test -p ralph-adapters cli_backend
   ```
   - Add test for `claude_tui()` method

4. **Integration**:
   ```bash
   cargo test
   ```
   - All existing tests should pass

## Complexity

Medium-High - Requires changes across multiple crates and careful coordination between TUI, PTY, and backend selection.

## Notes

- The key insight is that Claude without `-p` runs its own TUI, while `-p` mode outputs JSON that we parse
- For non-Claude backends, the existing behavior should remain unchanged
- The resize synchronization is critical for Claude's TUI to render correctly
