---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Code Task: Fix TUI Input Responsiveness

## Overview

Fix the TUI input lag where keypresses (like Ctrl+A for help) take several hundred milliseconds to respond. The current tick-driven event loop architecture causes input to be blocked by rendering.

## Root Cause

**Tick-driven event loop with render-then-poll ordering.**

The event loop in `crates/ralph-tui/src/app.rs` uses a 100ms tick interval where:
1. Every 100ms, the tick fires
2. Rendering happens FIRST (lines 133-171) while holding the state mutex
3. Input is polled SECOND (lines 173-234) with a 0ms timeout

This means:
- Input can only be processed at most 10 times per second
- If you press a key at 95ms into a tick cycle, you wait 5ms for the tick + rendering time + next tick for visual update
- Worst case: 200ms+ latency from keypress to visible response

```rust
// CURRENT (problematic) - app.rs:128-234
let mut tick = interval(Duration::from_millis(100));
loop {
    tokio::select! {
        _ = tick.tick() => {
            // RENDER FIRST (unknown duration)
            let state = self.state.lock().unwrap();
            terminal.draw(|f| { ... })?;
            drop(state);

            // POLL INPUT SECOND (only after render)
            if event::poll(Duration::from_millis(0))? { ... }
        }
    }
}
```

## Requirements

### 1. Restructure event loop to be event-driven

**File:** `crates/ralph-tui/src/app.rs`

Change from tick-driven to event-driven architecture where input polling is the primary blocking call:

```rust
// TARGET architecture (event-driven)
let mut last_render = Instant::now();
let render_interval = Duration::from_millis(16); // ~60fps cap

loop {
    tokio::select! {
        // Input polling is the PRIMARY driver with reasonable timeout
        result = poll_events(Duration::from_millis(16)) => {
            if let Some(event) = result? {
                // Process input IMMEDIATELY
                handle_event(event);
            }
        }
        _ = self.terminated_rx.changed() => {
            if *self.terminated_rx.borrow() {
                break;
            }
        }
    }

    // Render at controlled rate AFTER input processing
    if last_render.elapsed() >= render_interval {
        render(&mut terminal)?;
        last_render = Instant::now();
    }
}
```

### 2. Create async-compatible event polling

**File:** `crates/ralph-tui/src/app.rs` (or new `input.rs` module)

Crossterm's `event::poll()` is blocking. For proper async integration, use a spawned blocking task or crossterm's async features:

```rust
/// Poll for crossterm events in an async-compatible way
async fn poll_event_async(timeout: Duration) -> Result<Option<Event>> {
    // Option A: Use tokio's spawn_blocking
    tokio::task::spawn_blocking(move || {
        if event::poll(timeout)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }).await?
}
```

Alternative: Use crossterm's `EventStream` with tokio feature:
```rust
// In Cargo.toml: crossterm = { version = "...", features = ["event-stream"] }
use crossterm::event::EventStream;
use futures::StreamExt;

let mut event_stream = EventStream::new();

loop {
    tokio::select! {
        maybe_event = event_stream.next() => {
            if let Some(Ok(event)) = maybe_event {
                handle_event(event);
            }
        }
        // ... other branches
    }
}
```

### 3. Separate input handling from rendering

**File:** `crates/ralph-tui/src/app.rs`

Ensure input processing and rendering are decoupled:

```rust
// Track if UI needs redraw
let mut needs_redraw = true;

loop {
    // 1. Poll and process ALL pending input events
    while let Some(event) = poll_event_nonblocking()? {
        if handle_event(event, &mut state)? {
            needs_redraw = true;
        }
    }

    // 2. Render only when needed and at controlled rate
    if needs_redraw && last_render.elapsed() >= render_interval {
        render(&mut terminal, &state)?;
        needs_redraw = false;
        last_render = Instant::now();
    }

    // 3. Wait for next event or render tick
    tokio::select! {
        _ = poll_event_async(render_interval) => { ... }
        _ = self.terminated_rx.changed() => { ... }
    }
}
```

### 4. Preserve existing functionality

Ensure these features continue to work:
- Ctrl+C handling (signal interrupt_tx channel)
- Mouse scroll events
- Help overlay toggle (Ctrl+A)
- Search mode
- Termination signal from PTY
- Terminal cleanup on any exit path (defer! guard)

## Files to Modify

1. `crates/ralph-tui/src/app.rs` - Main event loop restructure
2. `crates/ralph-tui/Cargo.toml` - Add `event-stream` feature to crossterm if using EventStream approach

## Acceptance Criteria

- [ ] Keypress to visible response latency is under 50ms (target: 16ms)
- [ ] Ctrl+A shows help overlay within one frame (~16ms)
- [ ] Ctrl+C still properly signals main loop and exits
- [ ] Mouse scroll works smoothly
- [ ] Search mode works correctly
- [ ] PTY termination signal still exits TUI cleanly
- [ ] Terminal cleanup works on all exit paths
- [ ] CPU usage remains reasonable (no busy-spinning)
- [ ] All existing tests pass
- [ ] No clippy warnings

## Test Plan

### 1. Manual responsiveness test
```bash
cargo build --release
./target/release/ralph run --tui -c ralph.claude.yml -p "echo hello"
# Press Ctrl+A rapidly
# Help should appear/dismiss instantly (no perceptible delay)
```

### 2. Latency measurement (optional)
```bash
# If you want to measure, add debug timing:
# In handle_event: log timestamp when Ctrl+A received
# In render: log timestamp when help overlay drawn
# Difference should be <50ms
```

### 3. Existing functionality verification
- [ ] Ctrl+C exits cleanly
- [ ] Mouse scroll up/down works
- [ ] j/k keyboard scroll works
- [ ] Search with / works
- [ ] n/N for next/prev match works
- [ ] [ and ] for iteration navigation works
- [ ] q quits

### 4. Edge cases
- [ ] Rapid key mashing doesn't cause issues
- [ ] Long-running renders don't block input
- [ ] TUI exits cleanly on PTY termination

## Implementation Notes

### Approach Recommendation

Use crossterm's `EventStream` with tokio. This is the cleanest async integration:

```toml
# Cargo.toml
crossterm = { version = "0.28", features = ["event-stream"] }
```

```rust
use crossterm::event::EventStream;
use futures::StreamExt;

let mut events = EventStream::new();
let render_interval = Duration::from_millis(16);
let mut render_tick = interval(render_interval);

loop {
    tokio::select! {
        biased;  // Prioritize input over render tick

        maybe_event = events.next() => {
            match maybe_event {
                Some(Ok(event)) => handle_event(event)?,
                Some(Err(e)) => return Err(e.into()),
                None => break,  // Stream ended
            }
        }
        _ = render_tick.tick() => {
            render(&mut terminal)?;
        }
        _ = self.terminated_rx.changed() => {
            if *self.terminated_rx.borrow() {
                break;
            }
        }
    }
}
```

The `biased` keyword ensures input is always checked before render tick, maximizing responsiveness.

### Why Not Just Reduce Tick Interval?

Reducing from 100ms to 16ms would help but doesn't fix the fundamental issue:
- Still renders before polling (input blocked during render)
- Still doesn't process multiple pending events
- Adds unnecessary renders even when nothing changed

The event-driven approach is the correct architectural fix.

## Complexity

Medium - Requires restructuring the event loop but the logic is straightforward. Main risk is ensuring all existing functionality is preserved.

## Labels

TUI, Performance, Event Loop, User Experience
