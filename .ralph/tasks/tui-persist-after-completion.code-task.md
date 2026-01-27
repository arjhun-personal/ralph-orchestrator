---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Keep TUI Alive After Orchestration Completion

## Description
Modify the TUI lifecycle so it persists after the orchestration loop completes naturally, allowing users to browse iteration history, scroll through output, and search content until they manually exit with the 'q' key. Currently, the TUI exits immediately when the loop terminates for any reason.

## Background
The TUI is designed as a pure observer with no built-in exit logic - it runs indefinitely until explicitly signaled via `terminated_tx`. Currently, `cleanup_tui()` is called at all 7 termination points in `main.rs`, immediately signaling the TUI to exit regardless of whether the user pressed Ctrl+C or the loop completed naturally.

The TUI already displays a `■ done` indicator in the footer when `pending_hat` becomes `None` (via the `loop.terminate` event), so users can see when orchestration is complete. The missing piece is letting them continue browsing after completion.

**Current termination points in `main.rs`:**
| Line | Trigger | Should Keep TUI Alive? |
|------|---------|------------------------|
| 1718 | Interrupt at loop start (Ctrl+C) | No |
| 1732 | `check_termination()` (MaxIterationsReached) | Yes |
| 1762 | Fallback recovery exhausted | Yes |
| 1786 | No hats with pending events | Yes |
| 1922 | Interrupt during execution (Ctrl+C) | No |
| 1935 | Hat requests termination | Yes |
| 1971 | CompletionPromise detected | Yes |

## Technical Requirements
1. Distinguish between interrupt exits (Ctrl+C) and natural completions
2. For interrupt exits: keep current behavior - signal TUI to exit immediately
3. For natural completions: do NOT signal termination, instead await the TUI handle
4. Refactor the `cleanup_tui` closure to handle ownership correctly (currently moves `tui_handle`)
5. Ensure `loop.terminate` event is still published before waiting (TUI shows "done" indicator)
6. Maintain proper terminal cleanup on all exit paths

## Dependencies
- `crates/ralph-cli/src/main.rs` - Main orchestration loop with TUI setup and termination handling
- `crates/ralph-tui/src/app.rs` - TUI event loop (no changes needed, already supports 'q' exit)
- Understanding of tokio watch channels and JoinHandle awaiting

## Implementation Approach
1. **Refactor termination handling pattern:**
   - Replace or modify `cleanup_tui` closure (lines 1684-1690)
   - Create two distinct behaviors: `signal_tui_exit()` for interrupts, `wait_for_tui_exit()` for natural completions
   - Handle `tui_handle` ownership - consider using `Option::take()` pattern or restructuring

2. **Update interrupt exit paths (keep immediate exit):**
   - Line 1718: Interrupt at loop start
   - Line 1922: Interrupt during execution
   - These should call `terminated_tx.send(true)` and return immediately

3. **Update natural completion exit paths (wait for user):**
   - Line 1732: `check_termination()` returns reason
   - Line 1762: Fallback exhausted
   - Line 1786: No pending events
   - Line 1935: Hat requests termination
   - Line 1971: CompletionPromise detected
   - These should NOT signal termination, instead await the TUI handle

4. **Verify terminal cleanup:**
   - The TUI uses a `defer!` guard for cleanup - ensure this still works correctly
   - Test that terminal state is restored properly on all exit paths

## Acceptance Criteria

1. **Natural Completion Keeps TUI Alive**
   - Given the orchestration loop completes naturally (e.g., CompletionPromise detected)
   - When the loop terminates
   - Then the TUI remains visible and interactive, showing the `■ done` indicator

2. **User Can Browse After Completion**
   - Given the orchestration has completed and TUI is still running
   - When the user navigates iterations (h/l), scrolls (j/k), or searches (/)
   - Then all TUI interactions work normally

3. **User Exits with 'q' Key**
   - Given the TUI is running after completion
   - When the user presses 'q'
   - Then the TUI exits cleanly and terminal is restored

4. **Ctrl+C Still Exits Immediately**
   - Given the orchestration loop is running (or has completed)
   - When the user presses Ctrl+C
   - Then the TUI exits immediately without waiting for 'q'

5. **Terminal State Restored on All Exits**
   - Given any exit path (natural completion + 'q', or Ctrl+C interrupt)
   - When the TUI exits
   - Then the terminal is restored to normal mode (alternate screen disabled, raw mode off)

6. **Loop Terminate Event Still Published**
   - Given any termination reason
   - When the orchestration loop terminates
   - Then the `loop.terminate` event is published to observers (TUI shows done indicator)

## Metadata
- **Complexity**: Medium
- **Labels**: TUI, UX, Lifecycle, Terminal
- **Required Skills**: Rust async/await, tokio channels, terminal handling, ownership patterns
