---
status: implemented
gap_analysis: 2026-01-14
related:
  - event-loop.spec.md
---

# Terminal UI Spec

Ralph Orchestrator's real-time terminal dashboard for monitoring loop execution.

## Overview

A ratatui-based terminal UI that displays Ralph's current state during orchestration runs. The UI observes loop events in real-time using the Observer pattern, showing which hat Ralph is wearing, iteration progress, and timing information.

## Goals

1. **Visibility**: See Ralph's current activity without parsing log output
2. **Progress tracking**: Know iteration count and elapsed time at a glance
3. **Non-intrusive**: UI observes state; it doesn't control execution
4. **Minimal overhead**: Rendering shouldn't slow the orchestration loop

## Non-Goals

- Interactive controls (pause, resume, cancel) - future work
- Historical event browsing - use `ralph events` command
- Log streaming - separate concern
- Configuration editing - use YAML files

## Architecture

### Observer/State Pattern

The UI integrates via the existing `EventBus.set_observer()` callback mechanism. The observer receives all events as they flow through the system.

```
┌─────────────────┐     publishes      ┌─────────────────┐
│   Event Loop    │ ─────────────────▶ │    EventBus     │
└─────────────────┘                    └────────┬────────┘
                                                │
                                         observer callback
                                                │
                                                ▼
                                       ┌─────────────────┐
                                       │   TuiObserver   │
                                       │  (implements    │
                                       │   Fn(&Event))   │
                                       └────────┬────────┘
                                                │
                                          updates state
                                                │
                                                ▼
                                       ┌─────────────────┐
                                       │    TuiState     │
                                       │  (Arc<Mutex>)   │
                                       └────────┬────────┘
                                                │
                                           renders to
                                                │
                                                ▼
                                       ┌─────────────────┐
                                       │  Ratatui Frame  │
                                       └─────────────────┘
```

### State Structure

The UI maintains observable state derived from loop events:

**TuiState** holds:
- `pending_hat`: Which hat will process the next event (`HatId` + display name)
- `iteration`: Current iteration number (displayed as 1-indexed)
- `loop_started`: Timestamp when the loop began
- `iteration_started`: Timestamp when this iteration began
- `last_event`: Most recent event topic for activity indicator
- `last_event_at`: Timestamp of last event (for activity indicator)

**Note on iteration display:** The `LoopState.iteration` counter represents completed iterations (starts at 0, incremented after each iteration completes). The TUI displays `iteration + 1` during execution to show the current iteration number (1-indexed).

**Note on pending vs current:** The observer callback fires *before* events are routed to subscribers. The TUI cannot know which hat is currently executing—it can only infer which hat will handle the next event based on topic-to-subscription mappings. This "pending hat" is actually more useful for monitoring: it tells you what's coming next.

### Event-to-State Mapping

| Event Topic | State Update |
|-------------|--------------|
| `task.start` | Reset all state, set `loop_started`, set `pending_hat` to planner |
| `task.resume` | Set `loop_started`, set `pending_hat` to planner |
| `build.task` | Set `pending_hat` to builder, reset `iteration_started` |
| `build.done` | Set `pending_hat` to planner, increment `iteration` |
| `build.blocked` | Set `pending_hat` to planner |
| `loop.terminate` | Set activity indicator to "done" state |
| Any event | Update `last_event`, update `last_event_at` |

The TUI infers `pending_hat` from event topics using known subscription mappings:
- Events matching `task.*` → planner subscribes
- Events matching `build.task` → builder subscribes
- Events matching `build.done`, `build.blocked` → planner subscribes

For custom hats, the TUI must be initialized with the `HatRegistry` to look up subscriptions.

## UI Layout

```
┌─────────────────────────────────────────────────────────┐
│  🎩 RALPH ORCHESTRATOR                          [LIVE]  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│     Next Hat:      📋 Planner                           │
│                                                         │
│     Iteration:     3                                    │
│                                                         │
│     Loop Time:     00:05:23                             │
│     This Iteration: 00:01:47                            │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  Last: build.done                              ◉ active │
└─────────────────────────────────────────────────────────┘
```

### Layout Regions

1. **Header**: Title bar with live/done indicator
2. **Status Panel**: Next hat, iteration, timing
3. **Footer**: Last event topic, activity indicator

### Hat Display

Each hat displays with an emoji and name:

| Hat ID | Display |
|--------|---------|
| `planner` | 📋 Planner |
| `builder` | 🔨 Builder |
| Custom | 🎭 {name} |

### Timing Display

- **Loop Time**: `HH:MM:SS` since `task.start` or `task.resume`
- **This Iteration**: `HH:MM:SS` since current iteration began (resets when hat changes)

Times update every 100ms via a separate tick mechanism (not blocking on events).

### Activity Indicator

- `◉ active` (green): Event received in last 2 seconds
- `◯ idle` (dim): No events in last 2 seconds
- `■ done` (blue): Loop terminated

## Integration

### Invocation

```bash
# Run with TUI enabled
ralph run --tui
```

No configuration required. The TUI uses sensible defaults (100ms refresh rate). Configuration options may be added in future versions if needed.

### Implementation Location

All TUI code lives in the `ralph-tui` crate:

- `lib.rs` - Public API: `Tui::new()`, `Tui::run()`
- `state.rs` - `TuiState` and state management
- `observer.rs` - `TuiObserver` implementing the callback
- `widgets/` - Ratatui widget implementations
- `app.rs` - Main application loop and event handling

### Dependencies

Uses workspace dependencies already available:
- `ratatui` - Terminal UI framework
- `crossterm` - Terminal manipulation backend

## Acceptance Criteria

1. **Hat visibility**: UI displays pending hat name and emoji within 100ms of relevant event
2. **Iteration counter**: Shows correct iteration number (1-indexed), updates on `build.done`
3. **Loop timer**: Shows total elapsed time since loop start, updates every 100ms
4. **Iteration timer**: Shows elapsed time for current iteration, resets on hat change
5. **Activity indicator**: Shows green when events received in last 2s, dims otherwise
6. **Clean exit**: UI restores terminal state (raw mode, alternate screen) on termination or Ctrl+C
7. **No interference**: UI observation doesn't affect loop execution or event routing

## Future Considerations

- Standalone `ralph tui` command (attach to existing run via `.agent/events.jsonl`)
- Event log panel showing recent events
- Cost tracking display (using existing `LoopState.cumulative_cost`)
- Multiple hat tracking for concurrent execution
- Detachable TUI that can reconnect to running loops
