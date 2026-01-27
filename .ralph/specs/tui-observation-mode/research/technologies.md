# Technologies - TUI Observation Mode

## Core Dependencies

### Ratatui (TUI Framework)
- **Version**: Latest stable
- **Used for**: Terminal rendering, widgets, layout
- **Relevant APIs**:
  - `widgets::{Block, Borders, Paragraph}` - Widget rendering
  - `text::{Line, Span}` - Styled text
  - `style::{Color, Style}` - Visual styling
  - `Terminal` + `TestBackend` - Testing

### Crossterm (Terminal Backend)
- **Used for**: Key events, terminal control
- **Relevant APIs**:
  - `event::{KeyCode, KeyEvent, KeyModifiers}` - Input handling
  - `KeyEventKind::Press` - Press event detection

### Clap (CLI Parser)
- **Used for**: Argument parsing
- **Relevant APIs**:
  - `#[arg(short, long)]` - Flag definition
  - `conflicts_with` - Mutual exclusion
  - `hide = true` - Hidden/deprecated flags

## Integration Points

### ralph-adapters
Provides PTY control commands used by execution controls:
```rust
// Used in app.rs:256-259
ralph_adapters::pty_handle::ControlCommand::Skip
ralph_adapters::pty_handle::ControlCommand::Abort
```

These control commands will no longer be sent from TUI after removal, but the adapter types remain unchanged.

### ralph-proto
Provides event types:
```rust
use ralph_proto::{Event, HatId};
```

No changes needed.

## Libraries Available But Not Needed

- **tokio**: Already used for async. No additional async patterns needed for this change.
- **anyhow**: Error handling. No new error types needed (removing features).

## Testing Infrastructure

- `ratatui::backend::TestBackend` - Used for widget tests
- `Terminal::new(backend)` - Test terminal creation
- Buffer inspection via `buffer.content()` - Verify rendered output

Example from `header.rs:117-138`:
```rust
fn render_to_string_with_width(state: &TuiState, width: u16) -> String {
    let backend = TestBackend::new(width, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| {
        let widget = render(state, width);
        f.render_widget(widget, f.area());
    }).unwrap();
    // Extract rendered text from buffer
}
```
