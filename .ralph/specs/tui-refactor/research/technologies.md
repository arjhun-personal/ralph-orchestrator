# Technologies - TUI Refactor

## Current Dependencies

### ralph-tui (Cargo.toml)

```toml
# Core TUI framework
ratatui.workspace = true        # Terminal UI library
crossterm.workspace = true       # Cross-platform terminal control

# VT100 parsing (TO BE REMOVED)
tui-term = "0.3"                 # VT100 terminal emulator widget

# Utilities
tokio.workspace = true           # Async runtime
anyhow.workspace = true          # Error handling
tracing.workspace = true         # Logging
scopeguard.workspace = true      # RAII cleanup guards
```

### ralph-adapters (Cargo.toml)

```toml
# Stream handling dependencies
serde.workspace = true           # Serialization
serde_json.workspace = true      # JSON parsing

# Terminal markdown rendering (used by PrettyStreamHandler)
termimad.workspace = true        # Markdown -> terminal rendering

# PTY support (stays for non-TUI mode)
portable-pty.workspace = true    # PTY allocation
nix.workspace = true             # Unix system calls
vt100.workspace = true           # VT100 state machine (separate from tui-term)
crossterm.workspace = true       # Terminal control
strip-ansi-escapes.workspace = true  # ANSI removal
```

## Key Libraries

### ratatui

**Version:** workspace (check root Cargo.toml)

**Usage:**
- `ratatui::widgets::Paragraph` - Text display
- `ratatui::layout::{Layout, Constraint, Direction, Rect}` - Layout
- `ratatui::text::{Line, Span}` - Styled text
- `ratatui::style::{Style, Color, Modifier}` - Styling
- `ratatui::buffer::Buffer` - Direct buffer rendering
- `ratatui::Terminal` with `CrosstermBackend` - Terminal abstraction
- `ratatui::backend::TestBackend` - Testing

**Key patterns:**
```rust
// Stateless widget via function
pub fn render(state: &State) -> Paragraph<'static> { ... }

// Stateful widget via trait
impl Widget for MyWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) { ... }
}

// Terminal drawing
terminal.draw(|f| {
    f.render_widget(widget, chunks[0]);
})?;
```

### crossterm

**Version:** workspace

**Usage:**
- `crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers}` - Input events
- `crossterm::terminal::{enable_raw_mode, disable_raw_mode}` - Raw mode
- `crossterm::execute!` - Terminal commands
- `crossterm::style::{Color, SetForegroundColor, ResetColor}` - Colors (in PrettyStreamHandler)

**Event polling pattern:**
```rust
if event::poll(Duration::from_millis(0))? {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => { ... }
        Event::Mouse(mouse) => { ... }
        _ => {}
    }
}
```

### termimad

**Version:** workspace

**Usage in PrettyStreamHandler:**
```rust
let skin = MadSkin::default();
let rendered = self.skin.term_text(&self.text_buffer);
let _ = self.stdout.write(rendered.to_string().as_bytes());
```

**For TuiStreamHandler:** We need to convert markdown to ratatui `Line`/`Span` instead of ANSI strings.

**Options:**
1. Parse markdown manually (simple subset)
2. Use termimad's `MadSkin::styled_text()` (if available)
3. Use `pulldown-cmark` for proper markdown parsing
4. Start simple: just render as plain text (phase 1)

### tui-term (TO BE REMOVED)

**Current usage:**
```rust
// crates/ralph-tui/src/widgets/terminal.rs:1
use tui_term::vt100::Parser;

// crates/ralph-tui/src/app.rs:182
f.render_widget(tui_term::widget::PseudoTerminal::new(widget.parser().screen()), chunks[1]);
```

**Removal impact:**
- Delete `crates/ralph-tui/src/widgets/terminal.rs`
- Remove `tui_term` from Cargo.toml
- Replace with `ContentPane` widget

### tokio

**Usage:**
```rust
tokio::select! { ... }           // Concurrent event handling
interval(Duration::from_millis(100))  // Tick timer
tokio::spawn(async move { ... }) // Background tasks
tokio::sync::watch::channel()    // Termination signaling
tokio::sync::mpsc::unbounded_channel()  // PTY I/O
```

### serde_json

**Usage in StreamHandler:**
```rust
fn on_tool_call(&mut self, name: &str, id: &str, input: &serde_json::Value);
```

**Tool summary extraction:**
```rust
input.get("file_path")?.as_str()
input.get("command")?.as_str()
```

## New Dependencies Needed

### None Required

The refactor uses existing dependencies:
- `ratatui` for TUI rendering (already present)
- `serde_json` for tool input parsing (already in ralph-adapters)
- No markdown parsing needed initially - plain text first, enhance later

## Dependency Changes

### Remove from ralph-tui

```toml
# Remove
tui-term = "0.3"
```

### Keep in ralph-adapters

```toml
# Keep - used by PrettyStreamHandler and PTY
termimad.workspace = true
vt100.workspace = true
```

## Architecture Notes

### ratatui Line/Span Model

```rust
// Single styled span
let span = Span::styled("text", Style::default().fg(Color::Blue));

// Line from multiple spans
let line = Line::from(vec![
    Span::raw("prefix: "),
    Span::styled("styled", Style::default().fg(Color::Green)),
]);

// Static ownership for storing in buffers
let line: Line<'static> = Line::from(spans);
```

### Thread Safety

```rust
// Shared state pattern
state: Arc<Mutex<TuiState>>

// Accessing state
let state = self.state.lock().unwrap();
```

### Async/Await Pattern

```rust
pub async fn run(mut self) -> Result<()> {
    loop {
        tokio::select! {
            _ = tick.tick() => { ... }
            _ = channel.changed() => { ... }
        }
    }
}
```
