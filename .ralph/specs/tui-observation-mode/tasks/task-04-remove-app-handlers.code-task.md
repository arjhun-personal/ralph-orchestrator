---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Remove Command Handlers from App

## Description
Remove the `Pause`, `Skip`, and `Abort` command handlers from the TUI app event loop. Also remove the pause check that prevents input forwarding when paused.

## Background
The app.rs file handles commands routed by the input router. Currently it handles Pause (toggles LoopMode), Skip (sends ControlCommand::Skip to PTY), and Abort (sends ControlCommand::Abort). With LoopMode removed, these handlers reference deleted code and must be removed.

## Reference Documentation
**Required:**
- Design: specs/tui-observation-mode/design.md (Section 4.4)

**Additional References:**
- specs/tui-observation-mode/context.md (app architecture)
- specs/tui-observation-mode/plan.md (Step 4)

**Note:** You MUST read the design document before beginning implementation.

## Technical Requirements
1. Remove `Command::Pause` handler (lines 248-254) that toggles loop_mode
2. Remove `Command::Skip` handler (lines 255-257) that sends ControlCommand::Skip
3. Remove `Command::Abort` handler (lines 258-260) that sends ControlCommand::Abort
4. Remove pause check in PTY input forwarding (lines 232-240): delete the `is_paused` check
5. Simplify to always forward keyboard input to PTY

## Dependencies
- Task 03 (LoopMode removed) - this task fixes compile errors from that removal

## Implementation Approach
1. Read `crates/ralph-tui/src/app.rs` focusing on command handling
2. Remove the three match arms for Pause, Skip, Abort commands
3. Remove the is_paused check and always forward keyboard input
4. Verify `cargo check -p ralph-tui` now only has errors in header.rs (next task)

## Acceptance Criteria

1. **Pause handler removed**
   - Given app.rs is modified
   - When searching for "Command::Pause"
   - Then no matches are found

2. **Skip handler removed**
   - Given app.rs is modified
   - When searching for "Command::Skip"
   - Then no matches are found

3. **Abort handler removed**
   - Given app.rs is modified
   - When searching for "Command::Abort"
   - Then no matches are found

4. **Pause check removed**
   - Given app.rs is modified
   - When searching for "is_paused" or "LoopMode"
   - Then no matches are found

5. **Input always forwarded**
   - Given the PTY forwarding code is modified
   - When keyboard input arrives (not in scroll mode)
   - Then it is always sent to PTY (no pause check)

6. **Compile progress**
   - Given all changes are complete
   - When running `cargo check -p ralph-tui`
   - Then only header.rs compile errors remain

## Metadata
- **Complexity**: Medium
- **Labels**: tui, event-handling, removal
- **Required Skills**: Rust, async event loops
