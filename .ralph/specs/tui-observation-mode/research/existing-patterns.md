# Existing Patterns - TUI Observation Mode

## Input Routing Pattern

The TUI uses a clean state machine pattern for input routing in `crates/ralph-tui/src/input.rs`:

```rust
// input.rs:6-12 - Input mode state machine
pub enum InputMode {
    Normal,           // Forward keys to PTY
    AwaitingCommand,  // Waiting for prefix command
    Scroll,           // Vim-like scroll navigation
    Search,           // Search query input
}
```

**Pattern**: Prefix key (`Ctrl+a`) transitions to `AwaitingCommand`, then next key is interpreted as a command. This pattern should be preserved.

**Key routing (input.rs:78-87)**:
```rust
RouteResult::Command(match c {
    'q' => Command::Quit,
    '?' => Command::Help,
    'p' => Command::Pause,    // TO REMOVE
    'n' => Command::Skip,     // TO REMOVE
    'a' => Command::Abort,    // TO REMOVE
    '[' => Command::EnterScroll,
    _ => Command::Unknown,
})
```

## CLI Argument Pattern

The CLI uses clap's `#[arg]` macros with consistent patterns in `crates/ralph-cli/src/main.rs`:

```rust
// main.rs:280-287 - Current pattern
#[arg(short, long, conflicts_with = "autonomous")]
interactive: bool,

#[arg(short, long, conflicts_with = "interactive")]
autonomous: bool,

// main.rs:310-312 - Deprecated alias pattern
#[arg(long, hide = true)]  // hide = true for deprecated
tui: bool,
```

**Pattern**: The codebase uses `conflicts_with` for mutually exclusive flags. The `--tui` flag should become primary by removing `hide = true` and removing `interactive` entirely.

## State Management Pattern

State is centralized in `TuiState` struct (`crates/ralph-tui/src/state.rs`):

```rust
// state.rs:7-12 - LoopMode enum (TO REMOVE)
pub enum LoopMode {
    Auto,
    Paused,
}

// state.rs:32-33 - loop_mode field (TO REMOVE)
pub loop_mode: LoopMode,
```

**Pattern**: The crate exposes `LoopMode` publicly via `lib.rs:24`:
```rust
pub use state::{LoopMode, TuiState};
```

This public export will need to be removed when `LoopMode` is deleted.

## Command Handler Pattern

Command handlers follow a consistent pattern in `app.rs:242-271`:

```rust
match cmd {
    Command::Quit => break,
    Command::Help => {
        self.state.lock().unwrap().show_help = true;
    }
    Command::Pause => { ... }    // TO REMOVE
    Command::Skip => { ... }     // TO REMOVE
    Command::Abort => { ... }    // TO REMOVE
    Command::EnterScroll => { ... }
    Command::Unknown => {}
}
```

**Pattern**: Each command handler is a simple match arm. Removing handlers is straightforward.

## Test Pattern

Tests follow a consistent pattern in `input.rs`:

```rust
// input.rs:233-241 - Test pattern (TO REMOVE)
#[test]
fn pause_command_returns_p() {
    let mut router = InputRouter::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    router.route_key(prefix);

    let cmd = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
    assert_eq!(router.route_key(cmd), RouteResult::Command(Command::Pause));
}
```

Three tests to remove: `pause_command_returns_p`, `skip_command_returns_n`, `abort_command_returns_a`.

## Widget Rendering Pattern

Header widget (`crates/ralph-tui/src/widgets/header.rs:77-88`) displays mode:

```rust
let mode = if width > WIDTH_COMPRESS {
    match state.loop_mode {
        LoopMode::Auto => Span::styled("▶ auto", Style::default().fg(Color::Green)),
        LoopMode::Paused => Span::styled("⏸ paused", Style::default().fg(Color::Yellow)),
    }
} else {
    match state.loop_mode {
        LoopMode::Auto => Span::styled("▶", Style::default().fg(Color::Green)),
        LoopMode::Paused => Span::styled("⏸", Style::default().fg(Color::Yellow)),
    }
};
```

After removing `LoopMode`, this should show only the auto/running state (no paused variant needed).

## Documentation Pattern

Documentation consistently references `-i`/`--interactive`. Found 42 occurrences across 21 files, including:
- `README.md`, `AGENTS.md` - Primary docs
- `docs/**/*.md` - User documentation
- `tasks/**/*.md` - Task files
- `specs/**/*.md` - Specifications

**Pattern**: All should be updated atomically to use `--tui`.

## Behavior Spec Pattern

`specs/behaviors.yaml` contains verifiable CLI behaviors:

```yaml
# behaviors.yaml:31-33
- name: "-i/--interactive flag exists"
  type: cli-flag
  run: ${RALPH_BIN:-ralph} run --help | grep -qE '^\s+-i, --interactive'
```

These behavior tests must be updated to verify `--tui` flag instead.
