---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Remove Command Variants and Routing

## Description
Remove `Pause`, `Skip`, and `Abort` command variants from the input router. These execution controls allow users to interfere with Ralph's autonomous operation and must be removed for observation-only mode.

## Background
The TUI input router maps key presses to `Command` enum variants. The prefix commands (`Ctrl+a` then key) include `p`→Pause, `n`→Skip, `a`→Abort which send control signals to the PTY. These must be removed so pressing these keys does nothing.

## Reference Documentation
**Required:**
- Design: specs/tui-observation-mode/design.md (Section 4.2)

**Additional References:**
- specs/tui-observation-mode/context.md (input routing patterns)
- specs/tui-observation-mode/plan.md (Step 2)

**Note:** You MUST read the design document before beginning implementation.

## Technical Requirements
1. Remove `Pause`, `Skip`, `Abort` variants from `Command` enum (lines 19-21)
2. Remove match arms for `'p'`, `'n'`, `'a'` in prefix command routing (lines 82-84)
3. Remove associated unit tests: `pause_command_returns_p`, `skip_command_returns_n`, `abort_command_returns_a`

## Dependencies
- Task 01 (CLI arguments) should be complete first for clean git history, though not strictly required

## Implementation Approach
1. Read `crates/ralph-tui/src/input.rs` to understand current command structure
2. Remove the three enum variants from `Command`
3. Remove the three match arms from prefix key routing
4. Remove the three unit tests (lines 234-261)
5. Run `cargo test -p ralph-tui` to verify remaining tests pass

## Acceptance Criteria

1. **Command enum simplified**
   - Given the input module is compiled
   - When inspecting `Command` enum
   - Then only `Quit`, `Help`, `EnterScroll`, `Unknown` variants exist

2. **Pause key does nothing**
   - Given the router is in `AwaitingCommand` state after `Ctrl+a`
   - When user presses `p`
   - Then `Command::Unknown` is returned (key ignored)

3. **Skip key does nothing**
   - Given the router is in `AwaitingCommand` state after `Ctrl+a`
   - When user presses `n`
   - Then `Command::Unknown` is returned (key ignored)

4. **Abort key does nothing**
   - Given the router is in `AwaitingCommand` state after `Ctrl+a`
   - When user presses `a`
   - Then `Command::Unknown` is returned (key ignored)

5. **Remaining tests pass**
   - Given all changes are made
   - When running `cargo test -p ralph-tui`
   - Then all remaining input router tests pass (scroll, search, quit, help)

## Metadata
- **Complexity**: Low
- **Labels**: tui, input-handling, removal
- **Required Skills**: Rust, enum pattern matching
