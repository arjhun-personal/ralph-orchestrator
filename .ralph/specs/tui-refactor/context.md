# Implementation Context - TUI Refactor

## Summary

This document provides the implementation context for the TUI refactor, based on codebase exploration by the Explorer hat.

**Design document:** `specs/tui-refactor/design/detailed-design.md`
**Research findings:** `specs/tui-refactor/research/`

## Key Files to Modify

| File | Action | Purpose |
|------|--------|---------|
| `ralph-adapters/src/stream_handler.rs` | ADD | TuiStreamHandler implementing StreamHandler trait |
| `ralph-tui/src/state.rs` | MODIFY | Add IterationBuffer, refactor TuiState for iteration management |
| `ralph-tui/src/widgets/content.rs` | ADD | New ContentPane widget replacing TerminalWidget |
| `ralph-tui/src/widgets/header.rs` | MODIFY | Update format to `[iter N/M]`, add `[LIVE]`/`[REVIEW]` mode |
| `ralph-tui/src/widgets/footer.rs` | MODIFY | Add "▶ New: iter N" indicator |
| `ralph-tui/src/input.rs` | MODIFY | Simplify to direct key→action mapping |
| `ralph-tui/src/app.rs` | MODIFY | Simplified event loop using TuiStreamHandler |
| `ralph-tui/src/lib.rs` | MODIFY | Update public exports |
| `ralph-cli/src/main.rs` | MODIFY | Wire TuiStreamHandler when TUI enabled |

## Files to Remove

| File | Reason |
|------|--------|
| `ralph-tui/src/widgets/terminal.rs` | VT100 widget no longer needed |
| `ralph-tui/src/scroll.rs` | Scroll logic moves to IterationBuffer (partial removal) |

## Dependencies to Remove

```toml
# Remove from ralph-tui/Cargo.toml
tui-term = "0.3"
```

## Integration Points

### 1. StreamHandler Trait (Critical)

**Location:** `ralph-adapters/src/stream_handler.rs:127-151`

```rust
pub trait StreamHandler: Send {
    fn on_text(&mut self, text: &str);
    fn on_tool_call(&mut self, name: &str, id: &str, input: &serde_json::Value);
    fn on_tool_result(&mut self, id: &str, output: &str);
    fn on_error(&mut self, error: &str);
    fn on_complete(&mut self, result: &SessionResult);
}
```

TuiStreamHandler must implement this trait identically to PrettyStreamHandler for output parity.

### 2. CLI Integration Point

**Location:** `ralph-cli/src/main.rs:1594-1622`

```rust
// Current TUI wiring
let tui_handle = if enable_tui {
    let mut tui = Tui::new();
    // ... configure tui ...
    if let Some(ref mut executor) = pty_executor {
        let pty_handle = executor.handle();
        tui = tui.with_pty(pty_handle);  // Currently uses PTY
    }
    // ...
}
```

**Change needed:** Replace PTY handle with TuiStreamHandler. The executor will call `run_observe_streaming()` with the TuiStreamHandler instead of using the PTY's raw output.

### 3. Event Observer Pattern

**Location:** `ralph-tui/src/lib.rs:75-82`

```rust
pub fn observer(&self) -> impl Fn(&Event) + Send + 'static {
    let state = Arc::clone(&self.state);
    move |event: &Event| {
        if let Ok(mut s) = state.lock() {
            s.update(event);
        }
    }
}
```

This pattern continues - orchestrator events (task.start, build.done, etc.) update TuiState for header/footer display.

### 4. App Event Loop Structure

**Location:** `ralph-tui/src/app.rs:132-323`

Current structure:
```rust
loop {
    tokio::select! {
        _ = tick.tick() => {
            // 1. Check iteration change (clears terminal)
            // 2. Compute layout
            // 3. Draw widgets
            // 4. Poll input
        }
        _ = terminated_rx.changed() => break;
    }
}
```

New structure will be similar but simpler:
- No PTY output task
- No iteration counter synchronization
- Draw from IterationBuffer instead of VT100 parser

## Patterns to Follow

### Widget Rendering

```rust
// Stateless: return Paragraph
pub fn render(state: &TuiState, width: u16) -> Paragraph<'static>

// Stateful: implement Widget trait
impl Widget for ContentPane<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
}
```

### State Updates

```rust
// Event-driven updates
impl TuiState {
    pub fn update(&mut self, event: &Event) { ... }
}

// StreamHandler updates via Arc<Mutex<>>
impl TuiStreamHandler {
    fn append_to_buffer(&mut self, lines: Vec<Line<'static>>) {
        if let Ok(mut state) = self.state.lock() {
            // ...
        }
    }
}
```

### Testing

```rust
// Widget tests with TestBackend
fn render_to_string_with_width(state: &TuiState, width: u16) -> String {
    let backend = TestBackend::new(width, 1);
    // ...
}

// StreamHandler tests
#[test]
fn test_output_parity() {
    let mut pretty = PrettyStreamHandler::new(false);
    let mut tui = TuiStreamHandler::new(state.clone());
    // ... feed same events, compare output
}
```

## Constraints Discovered

### 1. Static Lifetimes for ratatui Lines

```rust
// IterationBuffer must own its lines
pub lines: Vec<Line<'static>>,
```

This means strings must be owned (not borrowed) when creating spans.

### 2. Thread Safety Requirements

```rust
// StreamHandler: Send required
pub trait StreamHandler: Send

// Shared state pattern
state: Arc<Mutex<TuiState>>
```

### 3. Layout Constraints

```rust
// Fixed header/footer, flexible content
Constraint::Length(1),  // Header
Constraint::Min(0),     // Content
Constraint::Length(1),  // Footer
```

### 4. Markdown Rendering Decision

**Option 1 (Recommended for Phase 1):** Plain text only
- Fastest to implement
- Matches PrettyStreamHandler's buffered text output

**Option 2 (Enhancement):** Parse markdown to styled spans
- Use `pulldown-cmark` or manual parsing
- Convert headers, bold, code blocks to styles

Start with Option 1, add markdown support as enhancement.

## Broken Windows

6 low-risk code smells identified in touched files. All will be naturally resolved during the refactor since the affected code is being redesigned. See `research/broken-windows.md` for details.

## Available Utilities

| Utility | Location | Purpose |
|---------|----------|---------|
| `format_tool_summary()` | stream_handler.rs:231-252 | Extract tool display info |
| `truncate()` | stream_handler.rs:258-269 | UTF-8 safe truncation |
| `ScrollManager` | scroll.rs | Scroll state (reuse search logic) |
| `SearchState` | scroll.rs:13-19 | Search matches tracking |

## Test Strategy Considerations

1. **Unit tests first:** Test IterationBuffer, TuiStreamHandler formatting, input mapping
2. **Integration tests:** Verify output parity between PrettyStreamHandler and TuiStreamHandler
3. **Visual validation:** Use `/tui-validate` skill with updated criteria

## Next Steps

1. **Planner** creates detailed test strategy and implementation order
2. **Task Writer** converts plan to code tasks
3. **Builder** implements using TDD (RED → GREEN → REFACTOR)
