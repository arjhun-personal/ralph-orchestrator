# Current Ralph TUI Architecture

## Core Structure

The TUI is implemented as an **observer-based system** that integrates with the event bus:

```
Event Loop → EventBus.publish() → Observer callback → TuiState → Rendering
```

**Key files:**
- `lib.rs` - Public API entry point
- `app.rs` - Main event loop and rendering engine
- `state.rs` - State management derived from events
- `widgets/` - Ratatui widget implementations (header, status, footer)

## Design Pattern: Observer/State

Pull model:
1. EventBus accepts observer closure via `set_observer()`
2. Every event triggers the observer callback
3. Observer updates `TuiState` (wrapped in `Arc<Mutex<>>`)
4. App loop reads state at 100ms intervals and renders

## Existing Widgets

1. **Header Widget**: Title + live/done status badge
2. **Status Widget**: Next hat, iteration, loop time, iteration time
3. **Footer Widget**: Last event topic, activity indicator

## Current Layout

```rust
[
    Constraint::Length(3),      // Header
    Constraint::Min(0),         // Status (flexible)
    Constraint::Length(3),      // Footer
]
```

## What's Reusable

- EventBus observer integration
- TuiState event-to-state mapping
- Widget framework
- App loop structure (tokio-based)
- Terminal state management (raw mode, alt screen)

## What Needs Redesign

1. **App Loop**: Need `tokio::select!` for PTY output + timer
2. **Input Handling**: Currently only Ctrl+C; need full keyboard routing
3. **Layout**: Add flexible space for PTY output
4. **New Widget**: `PtyWidget` for terminal frame rendering

## Integration Path

1. Keep current TUI intact (it's working)
2. Add PTY output panel below header
3. Implement input forwarding
4. Coordinate output redirection
