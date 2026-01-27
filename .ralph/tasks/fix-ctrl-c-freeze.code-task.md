---
status: completed
created: 2026-01-14
started: 2026-01-14
completed: 2026-01-15
---
# Code Task: Fix TUI Freeze After Double Ctrl+C

## Overview

Fix the TUI freeze that occurs after pressing Ctrl+C twice. The TUI becomes unresponsive and requires killing the terminal pane.

## Root Cause

**Channel closure race condition - TUI doesn't know when PTY dies.**

After double Ctrl+C:
1. PTY executor detects double-press and calls `terminate_child()`
2. PTY child process is killed (SIGTERM → SIGKILL)
3. PTY executor closes channels and exits its main loop
4. But the TUI has **no notification** that the PTY is dead
5. TUI continues its event loop, holding mutex locks during `terminal.draw()`
6. Terminal I/O may block waiting for PTY that no longer exists
7. The spawned output reader task exits silently when `output_rx` closes
8. Result: TUI is stuck in a frozen state

## Requirements

### 1. Add PTY death notification channel

**File:** `crates/ralph-adapters/src/pty_handle.rs`

Add a watch channel to signal PTY termination:

```rust
use tokio::sync::watch;

pub struct PtyHandle {
    pub output_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    pub input_tx: mpsc::UnboundedSender<Vec<u8>>,
    pub control_tx: mpsc::UnboundedSender<ControlCommand>,
    /// Signals when the PTY process has terminated
    pub terminated_rx: watch::Receiver<bool>,
}
```

### 2. Create termination channel in PTY executor

**File:** `crates/ralph-adapters/src/pty_executor.rs`

Create the watch channel and signal when PTY exits:

```rust
impl PtyExecutor {
    pub fn new(backend: CliBackend, config: PtyConfig) -> Self {
        let (output_tx, output_rx) = mpsc::unbounded_channel();
        let (input_tx, input_rx) = mpsc::unbounded_channel();
        let (control_tx, control_rx) = mpsc::unbounded_channel();
        let (terminated_tx, terminated_rx) = watch::channel(false);

        Self {
            // ... existing fields
            terminated_tx: Some(terminated_tx),
            terminated_rx: Some(terminated_rx),
        }
    }

    pub fn handle(&mut self) -> PtyHandle {
        PtyHandle {
            output_rx: self.output_rx.take().expect("handle() already called"),
            input_tx: self.input_tx.take().expect("handle() already called"),
            control_tx: self.control_tx.take().expect("handle() already called"),
            terminated_rx: self.terminated_rx.take().expect("handle() already called"),
        }
    }
}
```

Signal termination when PTY exits:

```rust
// At the end of run_interactive(), before returning:
if let Some(tx) = &self.terminated_tx {
    let _ = tx.send(true);
}
```

### 3. Listen for termination in TUI event loop

**File:** `crates/ralph-tui/src/app.rs`

Add the termination receiver to App and listen in the select loop:

```rust
pub struct App {
    // ... existing fields
    terminated_rx: watch::Receiver<bool>,
}

// In run():
loop {
    tokio::select! {
        _ = tick.tick() => {
            // ... existing tick handling
        }
        _ = tokio::signal::ctrl_c() => {
            break;
        }
        _ = self.terminated_rx.changed() => {
            if *self.terminated_rx.borrow() {
                // PTY has terminated, exit gracefully
                break;
            }
        }
    }
}
```

### 4. Reduce mutex lock scope during drawing

**File:** `crates/ralph-tui/src/app.rs`

Don't hold locks across `terminal.draw()`:

```rust
// BEFORE (locks held during draw):
let state = self.state.lock().unwrap();
let widget = self.terminal_widget.lock().unwrap();
terminal.draw(|f| { ... })?;

// AFTER (clone data, release locks, then draw):
let (header_data, screen_snapshot, footer_data, show_help) = {
    let state = self.state.lock().unwrap();
    let widget = self.terminal_widget.lock().unwrap();
    (
        state.clone(),  // Or just extract needed fields
        // For screen: need to handle differently - see note below
        state.clone(),
        state.show_help,
    )
};
// Locks released here
terminal.draw(|f| { ... })?;
```

**Note:** The `PseudoTerminal` widget needs the parser's screen reference. Options:
- Keep widget lock during draw (current behavior) but add termination check first
- Clone screen state (may be expensive)
- Use try_lock with timeout

### 5. Add timeout to event polling

**File:** `crates/ralph-tui/src/app.rs`

Use a non-zero timeout for event polling to allow periodic health checks:

```rust
// BEFORE:
if event::poll(Duration::from_millis(0))? {

// AFTER:
if event::poll(Duration::from_millis(10))? {
```

## Files to Modify

1. `crates/ralph-adapters/src/pty_handle.rs` - Add `terminated_rx` field
2. `crates/ralph-adapters/src/pty_executor.rs` - Create and signal termination channel
3. `crates/ralph-tui/src/app.rs` - Listen for termination, reduce lock scope

## Acceptance Criteria

- [ ] Double Ctrl+C terminates PTY and TUI exits cleanly
- [ ] Terminal returns to normal state (raw mode disabled, alternate screen exited)
- [ ] No freeze or hang after PTY termination
- [ ] Existing tests pass
- [ ] No clippy warnings

## Test Plan

1. **Manual double Ctrl+C test:**
   ```bash
   cargo build --release
   ./target/release/ralph run --tui -c ralph.claude.yml -p "Long running task"
   # Press Ctrl+C twice quickly
   ```
   - TUI should exit cleanly
   - Terminal should return to normal

2. **Test with various termination scenarios:**
   - Double Ctrl+C at different points in execution
   - Ctrl+\ (force kill)
   - PTY process crashes
   - Idle timeout

3. **Verify terminal cleanup:**
   - After exit, terminal should be in normal mode
   - Shell prompt should be visible
   - No cursor artifacts

## Complexity

Medium-High - Requires changes across multiple crates and careful channel management.

## Notes

- The freeze occurs because TUI has no way to know PTY died
- Watch channels are cheap and appropriate for one-time signals
- Mutex lock scope is a secondary issue but worth fixing for robustness
- `event::poll(0)` is non-blocking but doesn't allow for health checks
