# Existing Patterns - TUI Refactor

## Widget Pattern

All TUI widgets follow a consistent pattern using ratatui:

### Header/Footer (Stateless Widgets)

```rust
// crates/ralph-tui/src/widgets/header.rs:29-104
pub fn render(state: &TuiState, width: u16) -> Paragraph<'static> {
    let mut spans = vec![];
    // Build spans based on state and width
    // ...
    let line = Line::from(spans);
    Paragraph::new(line)  // No block wrapper - borderless single line
}
```

**Key conventions:**
- Accept `&TuiState` for read-only access to shared state
- Accept `width: u16` for responsive/progressive disclosure
- Return `Paragraph<'static>` for simple single-line widgets
- Use `Span::styled()` with `Style::default().fg(Color::X)` for colors
- No block wrapper for borderless "Minimal Chrome" design

### Footer (Stateful Widget)

```rust
// crates/ralph-tui/src/widgets/footer.rs:11-23
pub struct Footer<'a> {
    state: &'a TuiState,
    scroll_manager: &'a ScrollManager,
}

impl Widget for Footer<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        // Layout with Constraint::Fill for flexible spacing
        let chunks = Layout::horizontal([...]).split(area);
        Paragraph::new(left).render(chunks[0], buf);
        Paragraph::new(right).render(chunks[2], buf);
    }
}
```

**Key conventions:**
- Implement `Widget` trait for stateful widgets needing multiple render passes
- Use `Layout::horizontal` with `Constraint::Fill(1)` for flexible spacing
- Lifetime parameter `'a` for borrowed references

### Terminal Widget (VT100 - TO BE REPLACED)

```rust
// crates/ralph-tui/src/widgets/terminal.rs:3-14
pub struct TerminalWidget {
    parser: Parser,  // tui_term::vt100::Parser
}

impl TerminalWidget {
    pub fn process(&mut self, bytes: &[u8]) { ... }
    pub fn clear(&mut self) { ... }
    pub fn resize(&mut self, rows: u16, cols: u16) { ... }
}
```

This will be replaced by `ContentPane` that renders `IterationBuffer` lines.

## State Pattern

### TuiState Structure

```rust
// crates/ralph-tui/src/state.rs:8-39
pub struct TuiState {
    pub pending_hat: Option<(HatId, String)>,
    pub iteration: u32,
    pub prev_iteration: u32,
    pub loop_started: Option<Instant>,
    // ... more fields
}

impl TuiState {
    pub fn update(&mut self, event: &Event) { ... }
    pub fn get_pending_hat_display(&self) -> String { ... }
    pub fn is_active(&self) -> bool { ... }
}
```

**Key conventions:**
- Use `Option<Instant>` for timestamps
- Use `Option<(HatId, String)>` for optional display data with ID
- Provide computed getters (`get_*`, `is_*` methods)
- `update(&mut self, event: &Event)` for event-driven state changes

## StreamHandler Pattern

### Trait Definition

```rust
// crates/ralph-adapters/src/stream_handler.rs:127-151
pub trait StreamHandler: Send {
    fn on_text(&mut self, text: &str);
    fn on_tool_call(&mut self, name: &str, id: &str, input: &serde_json::Value);
    fn on_tool_result(&mut self, id: &str, output: &str);
    fn on_error(&mut self, error: &str);
    fn on_complete(&mut self, result: &SessionResult);
}
```

### PrettyStreamHandler (Reference Implementation)

```rust
// crates/ralph-adapters/src/stream_handler.rs:23-125
pub struct PrettyStreamHandler {
    stdout: io::Stdout,
    verbose: bool,
    text_buffer: String,  // Buffer for markdown batching
    skin: MadSkin,        // termimad markdown renderer
}

impl StreamHandler for PrettyStreamHandler {
    fn on_text(&mut self, text: &str) {
        self.text_buffer.push_str(text);  // Buffer, don't render immediately
    }

    fn on_tool_call(&mut self, name: &str, _id: &str, input: &Value) {
        self.flush_text_buffer();  // Flush before tool call
        // Format: ⚙️ [ToolName] summary
        // Use Color::Blue for tool name, Color::DarkGrey for summary
    }

    fn on_complete(&mut self, result: &SessionResult) {
        self.flush_text_buffer();  // Flush any remaining text
        // Show duration/cost/turns with Color::Green or Color::Red
    }
}
```

**Key conventions:**
- Buffer text for batch rendering (markdown parsing)
- Flush buffer before tool calls and on complete
- Use standard icons: `⚙️` for tools, `✓` for results, `✗` for errors
- `format_tool_summary()` helper extracts relevant field per tool type
- `truncate()` helper for long strings with UTF-8 safety

## App Event Loop Pattern

```rust
// crates/ralph-tui/src/app.rs:111-328
pub async fn run(mut self) -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    defer! {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, ...);
    }

    let mut tick = interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            _ = tick.tick() => {
                // Draw UI
                terminal.draw(|f| { ... })?;

                // Poll for input
                if event::poll(Duration::from_millis(0))? {
                    match event::read()? { ... }
                }
            }
            _ = self.terminated_rx.changed() => {
                if *self.terminated_rx.borrow() { break; }
            }
        }
    }
}
```

**Key conventions:**
- Use `scopeguard::defer!` for cleanup (not explicit cleanup after loop)
- 100ms tick interval for smooth UI updates
- `tokio::select!` for concurrent event handling
- Watch channel for PTY termination notification
- Poll-based input handling with zero timeout

## Input Router Pattern

```rust
// crates/ralph-tui/src/input.rs:37-138
pub struct InputRouter {
    mode: InputMode,
    prefix_key: KeyCode,
    prefix_modifiers: KeyModifiers,
}

pub fn route_key(&mut self, key: KeyEvent) -> RouteResult {
    match self.mode {
        InputMode::Normal => { ... }
        InputMode::Scroll => { ... }
        InputMode::Search => { ... }
    }
}
```

**Key conventions:**
- State machine with explicit modes
- Return enum variants for caller to handle actions
- Prefix key pattern (Ctrl+A) for modal commands (TO BE REMOVED)

## Testing Patterns

### Unit Tests with TestBackend

```rust
// crates/ralph-tui/src/widgets/header.rs:115-136
fn render_to_string_with_width(state: &TuiState, width: u16) -> String {
    let backend = TestBackend::new(width, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| {
        let widget = render(state, width);
        f.render_widget(widget, f.area());
    }).unwrap();

    let buffer = terminal.backend().buffer();
    buffer.content().iter().map(|cell| cell.symbol()).collect::<String>()
}
```

### Integration Tests

```rust
// crates/ralph-tui/tests/iteration_boundary.rs:8-22
fn simulate_events(events: Vec<Event>) -> (Arc<Mutex<TuiState>>, Arc<Mutex<TerminalWidget>>) {
    let state = Arc::new(Mutex::new(TuiState::new()));
    let widget = Arc::new(Mutex::new(TerminalWidget::new()));
    for event in events {
        state.lock().unwrap().update(&event);
    }
    (state, widget)
}
```

**Key conventions:**
- Test files in `tests/` directory for integration tests
- Use `#[cfg(test)] mod tests` inline for unit tests
- Test with `TestBackend` for widget rendering
- Create test helpers for common setup patterns

## Layout Pattern

```rust
// crates/ralph-tui/src/app.rs:155-162
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(1),  // Header: single line
        Constraint::Min(0),     // Content: flexible
        Constraint::Length(1),  // Footer: single line
    ])
    .split(frame_area);
```

**Key conventions:**
- Vertical layout with fixed header/footer, flexible content
- `Constraint::Length(1)` for single-line borderless widgets
- `Constraint::Min(0)` for flexible content area
