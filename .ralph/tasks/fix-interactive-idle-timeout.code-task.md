---
status: completed
created: 2026-01-14
started: 2026-01-14
completed: 2026-01-14
---
# Fix Interactive Mode Idle Timeout Behavior

## Problem

When running Ralph with `-i` (interactive/TUI mode), the idle timeout terminates the entire loop instead of progressing to the next iteration.

**Current behavior:**
```
Idle timeout fires → Loop exits immediately → "0 iterations"
```

**Expected behavior:**
```
Idle timeout fires → Process output → Parse events → Next iteration
```

## Root Cause

In `crates/ralph-cli/src/main.rs:1390-1392`, `IdleTimeout` returns `Some(TerminationReason::Stopped)` which causes early exit at lines 1268-1274 before output processing occurs:

```rust
ralph_adapters::TerminationType::IdleTimeout => {
    warn!("PTY idle timeout reached, terminating loop");
    Some(TerminationReason::Stopped)  // <-- This exits the loop!
}
```

## Solution

In interactive mode, `IdleTimeout` should signal "iteration complete" (return `None`) rather than "loop stopped". This allows:
1. Output to be processed
2. Events to be parsed (even if none found in TUI mode)
3. Fallback logic to inject `task.resume` if no events
4. Next iteration to start

## Implementation

**File:** `crates/ralph-cli/src/main.rs`

**Location:** Lines 1388-1398 in `execute_pty` function

**Change:**

```rust
// BEFORE
let termination = match pty_result.termination {
    ralph_adapters::TerminationType::Natural => None,
    ralph_adapters::TerminationType::IdleTimeout => {
        warn!("PTY idle timeout reached, terminating loop");
        Some(TerminationReason::Stopped)
    }
    ralph_adapters::TerminationType::UserInterrupt
    | ralph_adapters::TerminationType::ForceKill => {
        Some(TerminationReason::Interrupted)
    }
};

// AFTER
let termination = match pty_result.termination {
    ralph_adapters::TerminationType::Natural => None,
    ralph_adapters::TerminationType::IdleTimeout => {
        if interactive {
            // In interactive mode, idle timeout signals iteration complete,
            // not loop termination. Let output be processed for events.
            info!("PTY idle timeout in interactive mode, iteration complete");
            None
        } else {
            warn!("PTY idle timeout reached, terminating loop");
            Some(TerminationReason::Stopped)
        }
    }
    ralph_adapters::TerminationType::UserInterrupt
    | ralph_adapters::TerminationType::ForceKill => {
        Some(TerminationReason::Interrupted)
    }
};
```

## Behavior Matrix

| Mode | Termination Type | Result |
|------|------------------|--------|
| Autonomous | IdleTimeout | `Stopped` (exit loop - safety mechanism) |
| Interactive | IdleTimeout | `None` (continue - iteration complete) |
| Any | Natural | `None` (continue) |
| Any | UserInterrupt | `Interrupted` (exit loop) |
| Any | ForceKill | `Interrupted` (exit loop) |

## Flow After Fix

```
Interactive mode:
    ↓
Idle timeout fires
    ↓
termination = None
    ↓
Output processed (line 1280: log_events_from_output)
    ↓
process_output() called (line 1283)
    ↓
No events found → has_pending_events() = false
    ↓
Fallback logic injects task.resume (line 1161)
    ↓
Planner triggered → Next iteration starts
```

## Testing

1. **Manual test:**
   ```bash
   cargo run --bin ralph -- run --tui -c ralph.claude.yml -p "Create a hello world script"
   ```
   - Let Claude complete the task
   - Stop interacting
   - Wait for idle timeout
   - **Expected:** Loop progresses to iteration 2 (not "0 iterations")

2. **Verify autonomous mode unchanged:**
   ```bash
   cargo run --bin ralph -- run -c ralph.claude.yml -p "Create a hello world script"
   ```
   - If idle timeout fires in autonomous mode, loop should still exit with `Stopped`

## Acceptance Criteria

- [ ] Interactive mode: idle timeout allows iteration progression
- [ ] Autonomous mode: idle timeout still terminates loop (unchanged)
- [ ] `cargo test` passes
- [ ] No regression in existing behavior
