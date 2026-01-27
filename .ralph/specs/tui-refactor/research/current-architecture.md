# Current Architecture Research

## Overview

This document captures research on the current TUI and non-TUI output systems to inform the refactor design.

## TUI Architecture (ralph-tui crate)

### Module Structure

| Module | File | Responsibility |
|--------|------|----------------|
| **lib** | `crates/ralph-tui/src/lib.rs` | Public API: `Tui` struct |
| **app** | `crates/ralph-tui/src/app.rs` | Main event loop, rendering, input handling |
| **state** | `crates/ralph-tui/src/state.rs` | Observable state from event bus |
| **input** | `crates/ralph-tui/src/input.rs` | Input routing, prefix commands (Ctrl+A) |
| **scroll** | `crates/ralph-tui/src/scroll.rs` | Scroll/search in terminal history |
| **widgets/** | `crates/ralph-tui/src/widgets/` | Ratatui components |

### Widgets

| Widget | File | Purpose |
|--------|------|---------|
| **TerminalWidget** | `widgets/terminal.rs` | VT100 parser via `tui-term`, maintains scrollback |
| **Header** | `widgets/header.rs` | Iteration, elapsed time, current hat, mode indicator |
| **Footer** | `widgets/footer.rs` | Activity indicator (◉/◯/■), last event, search state |
| **Help** | `widgets/help.rs` | Command reference overlay |

### Current Data Flow

```
Claude PTY → mpsc channel → Background task → TerminalWidget::process()
                                                      ↓
                                             VT100 parser (tui-term)
                                                      ↓
                                             Screen + scrollback buffer
                                                      ↓
                                             App::run() render loop (100ms)
                                                      ↓
                                             PseudoTerminal widget
```

### Current Interaction Model

The TUI has three input modes:
- **Normal**: Forward all keys to PTY (interactive)
- **Command**: Ctrl+A prefix interprets next key
- **Scroll**: Navigate history, search with /

This interaction model exists but the user wants to **remove** agent interaction.

---

## Non-TUI Architecture (ralph-adapters crate)

### StreamHandler Trait

```rust
pub trait StreamHandler: Send {
    fn on_text(&mut self, text: &str);
    fn on_tool_call(&mut self, name: &str, id: &str, input: &serde_json::Value);
    fn on_tool_result(&mut self, id: &str, output: &str);
    fn on_error(&mut self, error: &str);
    fn on_complete(&mut self, result: &SessionResult);
}
```

### Handler Implementations

| Handler | Usage | Features |
|---------|-------|----------|
| **PrettyStreamHandler** | TTY output | Markdown via termimad, colors, emoji icons |
| **ConsoleStreamHandler** | Piped output | Plain text, no ANSI |
| **QuietStreamHandler** | CI/scripting | Suppresses all output |

### PrettyStreamHandler Output Format

| Event | Format | Color | Icon |
|-------|--------|-------|------|
| Text | Markdown rendered | Default | - |
| Tool Call | `⚙️ [ToolName] summary` | Blue | ⚙️ |
| Tool Result | `✓ output...` | Dark Grey | ✓ |
| Error | `✗ Error: message` | Red | ✗ |
| Complete | `Duration/Cost/Turns` | Green/Red | - |

### Non-TUI Data Flow

```
Claude JSON Stream → ClaudeStreamParser::parse_line()
                              ↓
                    ClaudeStreamEvent
                              ↓
                    dispatch_stream_event(handler)
                              ↓
                    PrettyStreamHandler methods
                              ↓
                    termimad markdown → terminal
```

---

## Key Architectural Difference

| Aspect | TUI Mode | Non-TUI Mode |
|--------|----------|--------------|
| **Input Source** | Raw PTY bytes | Parsed JSON stream |
| **Parsing** | VT100 escape codes | Structured JSON events |
| **Rendering** | `tui-term` PseudoTerminal | `termimad` markdown |
| **Output** | PTY terminal emulator | Direct to stdout |

**Problem**: TUI receives raw PTY output with ANSI codes, while non-TUI gets structured JSON events. These are fundamentally different pipelines.

---

## Files for Reference

- TUI crate: `crates/ralph-tui/src/`
- Stream handlers: `crates/ralph-adapters/src/stream_handler.rs`
- PTY executor: `crates/ralph-adapters/src/pty_executor.rs`
- Claude stream parser: `crates/ralph-adapters/src/claude_stream.rs`
- CLI integration: `crates/ralph-cli/src/main.rs:1594-1625` (TUI), `2014-2047` (handlers)
