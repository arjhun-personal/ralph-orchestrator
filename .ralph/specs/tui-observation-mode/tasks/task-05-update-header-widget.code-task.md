---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Update Header Widget

## Description
Update the header widget to always show "auto" mode indicator (no paused state) and remove the `LoopMode` dependency. Also remove the paused mode test.

## Background
The header widget displays the current loop mode as either "▶ auto" or "⏸ paused" based on `state.loop_mode`. With LoopMode removed, the header should always show "▶ auto" (or just "▶" in compressed mode).

## Reference Documentation
**Required:**
- Design: specs/tui-observation-mode/design.md (Section 4.5, implied)

**Additional References:**
- specs/tui-observation-mode/context.md (widget patterns)
- specs/tui-observation-mode/plan.md (Step 5)

**Note:** You MUST read the design document before beginning implementation.

## Technical Requirements
1. Remove `LoopMode` import from header.rs
2. Simplify mode display (lines 75-88): remove match on state.loop_mode
3. Always render "▶ auto" for full mode, "▶" for compressed mode
4. Update `create_full_state()` test helper to remove `loop_mode` assignment
5. Remove `header_shows_paused_mode` test (lines 233-244)

## Dependencies
- Task 04 (app handlers removed) - all LoopMode references should now be isolated to header

## Implementation Approach
1. Read `crates/ralph-tui/src/widgets/header.rs`
2. Remove the LoopMode import
3. Find the mode rendering code and replace match expression with hardcoded auto display
4. Update test helper `create_full_state()` to not set loop_mode
5. Delete the paused mode test
6. Run `cargo test -p ralph-tui` - all tests should now pass

## Acceptance Criteria

1. **LoopMode import removed**
   - Given header.rs is modified
   - When searching for "LoopMode" import
   - Then no matches are found

2. **Mode always shows auto**
   - Given the header widget renders
   - When displaying mode indicator
   - Then it always shows "▶ auto" (or "▶" compressed), never "⏸ paused"

3. **Test helper updated**
   - Given `create_full_state()` is modified
   - When inspecting the helper function
   - Then no `loop_mode` field assignment exists

4. **Paused test removed**
   - Given header.rs tests are modified
   - When searching for "paused_mode"
   - Then no test function matches

5. **All tests pass**
   - Given all changes are complete
   - When running `cargo test -p ralph-tui`
   - Then all tests pass (full compilation restored)

## Metadata
- **Complexity**: Medium
- **Labels**: tui, widget, rendering
- **Required Skills**: Rust, ratatui widgets
