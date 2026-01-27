---
status: completed
created: 2026-01-14
started: 2026-01-14
completed: 2026-01-14
---
# Code Task: Fix Terminal State Corruption on TUI Abort

## Overview

After double Ctrl+C termination, the terminal is left in a broken state (raw mode, alternate screen) requiring the user to force-kill their terminal pane. This is caused by the TUI cleanup code being skipped when the task is aborted.

## Root Cause Analysis

**Two bugs working together:**

### Bug 1: TUI Task Aborted Without Cleanup

**File:** `crates/ralph-cli/src/main.rs:1117-1121`

```rust
let cleanup_tui = |tui_handle: Option<tokio::task::JoinHandle<Result<()>>>| {
    if let Some(handle) = tui_handle {
        handle.abort();  // ⚠️ Kills task immediately!
    }
};
```

When `handle.abort()` is called, the TUI task is cancelled at whatever await point it's at. The cleanup code at the end of `app.rs:run()` **never executes**:

```rust
// app.rs:331-336 - SKIPPED on abort!
disable_raw_mode()?;
execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
```

**Result:** Terminal left in raw mode + alternate screen = appears frozen.

### Bug 2: Blocking Sleep in Async Context

**File:** `crates/ralph-adapters/src/pty_executor.rs:1127-1134`

```rust
while start.elapsed() < grace_period {
    if child.try_wait()...
    std::thread::sleep(Duration::from_millis(100));  // ⚠️ Blocks tokio worker!
}
```

The `terminate_child()` function uses `std::thread::sleep()` inside an `async fn`. This blocks the tokio worker thread for up to 5 seconds during graceful termination, making the TUI appear frozen.

## Requirements

### 1. Fix TUI Cleanup - Add Terminal State Guard

**File:** `crates/ralph-tui/src/app.rs`

Use a `Drop` guard to ensure terminal cleanup happens on ANY exit path (including task abort):

```rust
use scopeguard::defer;

pub async fn run(mut self) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Ensure terminal cleanup on ANY exit (including abort/panic)
    defer! {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            crossterm::cursor::Show
        );
    }

    // ... rest of run() loop ...

    // Note: The explicit cleanup at the end is now redundant but harmless
    // (disable_raw_mode is idempotent)
}
```

### 2. Fix cleanup_tui - Graceful Shutdown Instead of Abort

**File:** `crates/ralph-cli/src/main.rs`

Option A: Signal termination and wait for graceful exit:

```rust
let cleanup_tui = |tui_handle: Option<tokio::task::JoinHandle<Result<()>>>,
                   terminated_tx: &watch::Sender<bool>| {
    // Signal TUI to exit
    let _ = terminated_tx.send(true);

    if let Some(handle) = tui_handle {
        // Give TUI time to clean up gracefully
        let rt = tokio::runtime::Handle::current();
        let _ = rt.block_on(async {
            tokio::time::timeout(Duration::from_millis(500), handle).await
        });
    }
};
```

Option B: Keep abort but rely on the Drop guard (simpler, preferred with Requirement 1):

```rust
// With the scopeguard in place, abort is now safe
let cleanup_tui = |tui_handle: Option<tokio::task::JoinHandle<Result<()>>>| {
    if let Some(handle) = tui_handle {
        handle.abort();
        // Drop guard in TUI ensures terminal is restored
    }
};
```

### 3. Replace Blocking Sleep with Async Sleep

**File:** `crates/ralph-adapters/src/pty_executor.rs`

Replace the synchronous sleep in `terminate_child()`:

```rust
// BEFORE (blocking):
fn terminate_child(&self, child: &mut Box<dyn portable_pty::Child + Send>, graceful: bool) -> io::Result<()> {
    // ...
    while start.elapsed() < grace_period {
        if child.try_wait()...
        std::thread::sleep(Duration::from_millis(100));  // Blocks!
    }
}

// AFTER (async):
async fn terminate_child(&self, child: &mut Box<dyn portable_pty::Child + Send>, graceful: bool) -> io::Result<()> {
    // ...
    while start.elapsed() < grace_period {
        if child.try_wait()...
        tokio::time::sleep(Duration::from_millis(100)).await;  // Non-blocking!
    }
}
```

**Note:** This requires updating all call sites to `.await` the result.

### 4. Alternative: Use spawn_blocking for Termination

If making `terminate_child` async is too invasive:

```rust
fn terminate_child(&self, child: &mut Box<dyn portable_pty::Child + Send>, graceful: bool) -> io::Result<()> {
    let pid = match child.process_id() {
        Some(id) => Pid::from_raw(id as i32),
        None => return Ok(()),
    };

    if graceful {
        let _ = kill(pid, Signal::SIGTERM);

        // Use non-blocking polling instead of sleep
        let grace_period = Duration::from_secs(5);
        let start = Instant::now();

        while start.elapsed() < grace_period {
            if child.try_wait()
                .map_err(|e| io::Error::other(e.to_string()))?
                .is_some()
            {
                return Ok(());
            }
            // Yield to tokio instead of blocking
            std::hint::spin_loop();
            // Or use a shorter sleep (less blocking but still not ideal)
        }
    }

    let _ = kill(pid, Signal::SIGKILL);
    Ok(())
}
```

## Files to Modify

1. `crates/ralph-tui/src/app.rs` - Add scopeguard for terminal cleanup
2. `crates/ralph-cli/src/main.rs` - Update cleanup_tui (optional if using guard)
3. `crates/ralph-adapters/src/pty_executor.rs` - Fix blocking sleep
4. `Cargo.toml` (ralph-tui) - Add scopeguard dependency if not present

## Acceptance Criteria

- [ ] Double Ctrl+C terminates without leaving terminal in broken state
- [ ] Terminal returns to normal mode (raw mode disabled, alternate screen exited)
- [ ] No 5-second freeze during termination (blocking sleep fixed)
- [ ] Force-killing zellij/tmux pane is NOT required after exit
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes with no warnings
- [ ] Manual testing confirms clean exit in all scenarios

## Test Plan

### 1. Double Ctrl+C Test

```bash
cargo build --release
./target/release/ralph run --tui -c ralph.claude.yml -p "Hello"
# Wait for Claude to start responding
# Press Ctrl+C twice quickly
```

**Expected:**
- TUI exits within 1 second (no 5-second freeze)
- Terminal returns to normal shell prompt
- No visual artifacts or broken state

### 2. Abort Timing Test

```bash
# Test abort at different points:
# - During iteration start
# - While Claude is thinking
# - While Claude is writing
# - During tool execution
```

### 3. Terminal State Verification

After each termination method, verify:
```bash
# Should see normal prompt, not raw mode artifacts
echo "Terminal is working"
# Cursor should be visible
# Scrollback should work
```

### 4. Stress Test

```bash
for i in {1..10}; do
    timeout 5 ./target/release/ralph run --tui -c ralph.claude.yml -p "Hi" &
    sleep 1
    kill -INT $!
    sleep 0.5
    kill -INT $!
    wait
done
echo "All iterations completed - terminal should be normal"
```

## Complexity

Medium - Requires understanding of async/await patterns and terminal state management. The scopeguard solution is straightforward; the async sleep fix requires careful call-site updates.

## Dependencies

- `scopeguard` crate (may already be in workspace)

## Notes

- The existing panic hook (`install_panic_hook()`) handles panics but not task aborts
- `scopeguard::defer!` works with both normal exits and task cancellation
- The 5-second grace period for SIGTERM is reasonable but the blocking sleep is not
- Consider reducing grace period to 2 seconds for faster user experience
