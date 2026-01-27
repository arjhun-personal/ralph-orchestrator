---
status: completed
created: 2026-01-14
started: 2026-01-14
completed: 2026-01-15
---
# Code Task: Fix Iteration Boundary Rendering Bug

## Overview

Fix the TUI rendering corruption that occurs after iteration boundaries. The terminal displays garbled/mixed content when transitioning between iterations.

## Root Cause

**Race condition between PTY output background task and iteration clear operation.**

The background task (spawned at app.rs:62-69) continuously processes PTY output bytes. When an iteration ends:
1. `iteration_changed()` returns true
2. `widget.clear()` creates a fresh parser
3. But orphaned bytes from the old PTY (still in the channel) are processed by the background task
4. These old bytes feed into the new empty parser, causing corruption

```
Timeline:
T0: build.done event → state.iteration incremented
T1: Main loop tick → widget.clear() called → Parser is EMPTY
T1.5: PTY OUTPUT TASK receives ORPHANED BYTES from old iteration
      → Processes them into the fresh parser
      → Parser now contains invalid content
T2: terminal.draw() renders mixed/garbled content
```

## Requirements

### 1. Add iteration synchronization flag

**File:** `crates/ralph-tui/src/app.rs`

Add an atomic flag to coordinate between the main loop and the output reader task:

```rust
use std::sync::atomic::{AtomicU32, Ordering};

pub struct App {
    // ... existing fields
    iteration_counter: Arc<AtomicU32>,
}
```

### 2. Pass iteration counter to output reader task

**File:** `crates/ralph-tui/src/app.rs`

Modify the spawned output task to track iteration boundaries:

```rust
// In with_prefix():
let iteration_counter = Arc::new(AtomicU32::new(0));
let iteration_for_task = Arc::clone(&iteration_counter);

tokio::spawn(async move {
    let mut last_seen_iteration = 0;
    while let Some(bytes) = output_rx.recv().await {
        let current = iteration_for_task.load(Ordering::Acquire);
        if current != last_seen_iteration {
            // Iteration boundary - skip orphaned bytes from old iteration
            last_seen_iteration = current;
            continue;
        }
        if let Ok(mut widget) = widget_clone.lock() {
            widget.process(&bytes);
        }
    }
});
```

### 3. Increment counter on iteration change

**File:** `crates/ralph-tui/src/app.rs`

When clearing the widget, also increment the iteration counter:

```rust
if state.iteration_changed() {
    state.prev_iteration = state.iteration;
    drop(state);

    // Signal iteration boundary to output task BEFORE clearing
    self.iteration_counter.fetch_add(1, Ordering::Release);

    let mut widget = self.terminal_widget.lock().unwrap();
    widget.clear();
    self.scroll_manager.reset();
}
```

### 4. Alternative: Drain channel before clear

If the atomic approach is too complex, a simpler fix is to briefly pause and drain:

```rust
if state.iteration_changed() {
    state.prev_iteration = state.iteration;
    drop(state);

    // Small delay to let pending bytes arrive
    tokio::time::sleep(Duration::from_millis(10)).await;

    let mut widget = self.terminal_widget.lock().unwrap();
    widget.clear();
    self.scroll_manager.reset();
}
```

## Files to Modify

1. `crates/ralph-tui/src/app.rs` - Add iteration synchronization

## Acceptance Criteria

- [ ] Terminal displays clean content after iteration transitions
- [ ] No garbled/mixed output from previous iterations
- [ ] Existing tests pass (`cargo test -p ralph-tui`)
- [ ] No clippy warnings
- [ ] Performance not significantly impacted (atomic operations are cheap)

## Test Plan

1. **Manual test with multiple iterations:**
   ```bash
   cargo build --release
   ./target/release/ralph run --tui -c ralph.claude.yml -p "Do task 1, then task 2"
   ```
   - Watch for clean transitions between iterations
   - No garbled text should appear

2. **Stress test with rapid iterations:**
   - Configure short idle timeout
   - Force rapid iteration changes
   - Verify rendering stays clean

## Complexity

Medium - Requires careful synchronization between concurrent tasks.

## Notes

- The CLAUDE.md tenet "Fresh Context Is Reliability" applies here: each iteration should start with completely clean state
- The race window is 0-100ms (the tick interval)
- Using `Ordering::Acquire/Release` for proper memory synchronization
