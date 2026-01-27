---
status: completed
created: 2025-01-19
started: 2025-01-19
completed: 2025-01-19
---
# Task: Add ANSI Color Preservation for Non-Claude Backend Output in TUI Mode

## Description
Enable TUI mode to display colored terminal output from non-Claude backends (Kiro, Gemini, Codex, etc.). Currently, when these backends emit ANSI-colored output, it is either not streamed to the TUI at all (due to batch mode fallback) or loses color information (due to markdown-only parsing). This task fixes both issues to provide a rich, colored TUI experience for all backends.

## Background
The TUI streaming architecture was originally designed for Claude's NDJSON (`StreamJson`) output format. Two issues prevent non-Claude backends from displaying properly:

1. **Streaming bypass**: In `pty_executor.rs:543-549`, when `OutputFormat::Text` is detected, `run_observe_streaming()` immediately falls back to `run_observe()` (batch mode), which never calls the stream handler. The TUI receives no output during execution.

2. **ANSI code loss**: Even if streaming worked, `TuiStreamHandler` passes text through `tui_markdown::from_str()` which doesn't understand ANSI escape codes. Colors are lost.

### Key Files
- `crates/ralph-adapters/src/pty_executor.rs` - PTY execution and streaming logic
- `crates/ralph-adapters/src/stream_handler.rs` - Stream handler and Line conversion
- `crates/ralph-adapters/src/cli_backend.rs` - Backend definitions with `OutputFormat`

### Current Data Flow (Broken for Text backends)
```
Kiro CLI → PTY Output (has ANSI)
    ↓
run_observe_streaming() detects OutputFormat::Text
    ↓
Falls back to run_observe() (batch mode)
    ↓
Handler NEVER called → TUI shows nothing
```

### Target Data Flow
```
Kiro CLI → PTY Output (has ANSI)
    ↓
run_observe_streaming() handles Text format
    ↓
Streams chunks to handler.on_text()
    ↓
ANSI detection → ansi_to_tui parsing
    ↓
Styled ratatui Lines → TUI displays colors ✓
```

## Technical Requirements
1. Extend `run_observe_streaming()` to handle `OutputFormat::Text` by streaming raw output chunks to the handler instead of falling back to batch mode
2. Add `ansi-to-tui` crate (v8.0.1+) as a dependency to `ralph-adapters`
3. Modify `stream_handler.rs` to detect ANSI escape codes in incoming text
4. When ANSI codes are present, use `ansi_to_tui` to parse into styled `Line<'static>` objects
5. When no ANSI codes are present, continue using `tui_markdown` for markdown rendering
6. Preserve existing Claude/StreamJson functionality unchanged

## Dependencies
- `ansi-to-tui` crate (v8.0.1+) - Official ratatui-adjacent project for ANSI → ratatui conversion
- Existing `ratatui` (v0.30) - Already in use
- Existing `strip-ansi-escapes` (v0.2) - Already used for event parsing, can help with detection

## Implementation Approach

### Part 1: Enable Text Backend Streaming
1. In `pty_executor.rs`, modify `run_observe_streaming()` to NOT early-return for `OutputFormat::Text`
2. Add a text-streaming code path that reads PTY output in chunks
3. Call `handler.on_text()` for each chunk of raw output (preserving ANSI codes)
4. Ensure proper handling of partial lines and buffering

### Part 2: Add ANSI Parsing to Stream Handler
1. Add `ansi-to-tui = "8"` to `ralph-adapters/Cargo.toml`
2. Create a helper function to detect ANSI escape codes (check for `\x1b[` or `\033[`)
3. Modify `markdown_to_lines()` or create `text_to_lines()` function:
   - If ANSI codes detected: use `ansi_to_tui::IntoText` trait
   - If no ANSI codes: use existing `tui_markdown::from_str()`
4. Update `TuiStreamHandler.update_lines()` to use the new parsing logic

### Part 3: Testing
1. Test with Kiro backend to verify colored output displays
2. Test with Claude backend to verify no regression in markdown rendering
3. Verify ANSI codes for: foreground colors, background colors, bold, italic, underline, reset

## Acceptance Criteria

1. **Text Backend Streaming Enabled**
   - Given a backend with `OutputFormat::Text` (e.g., Kiro)
   - When running in TUI mode (`--tui`)
   - Then output appears in the TUI in real-time during execution (not just after completion)

2. **ANSI Colors Preserved**
   - Given backend output containing ANSI color codes (e.g., `\x1b[32mgreen\x1b[0m`)
   - When displayed in the TUI
   - Then the text renders with the correct colors and styling

3. **ANSI Modifiers Supported**
   - Given backend output with ANSI modifiers (bold, italic, underline, dim)
   - When displayed in the TUI
   - Then the text renders with the correct style modifiers

4. **Claude Backend Unchanged**
   - Given the Claude backend with `OutputFormat::StreamJson`
   - When running in TUI mode
   - Then markdown formatting continues to work correctly (no regression)

5. **Plain Text Fallback**
   - Given backend output with no ANSI codes and no markdown
   - When displayed in the TUI
   - Then text renders as plain unstyled text

6. **Mixed Content Handling**
   - Given output that contains both ANSI codes and plain text sections
   - When displayed in the TUI
   - Then each section renders appropriately (styled vs unstyled)

7. **Unit Tests**
   - Given the new ANSI detection and parsing logic
   - When running `cargo test`
   - Then unit tests verify ANSI detection, parsing, and Line conversion

## Metadata
- **Complexity**: Medium
- **Labels**: TUI, Streaming, ANSI, Colors, Backend Support, Enhancement
- **Required Skills**: Rust, ratatui, PTY handling, ANSI escape codes, async streaming
