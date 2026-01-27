---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Remove LoopMode Enum and State Field

## Description
Remove the `LoopMode` enum and its usage from TUI state. This enum tracked whether Ralph was paused or running auto. With pause functionality removed, this state is unnecessary (YAGNI).

## Background
`LoopMode` has two variants: `Auto` and `Paused`. It's stored in `TuiState.loop_mode` and exported publicly from `ralph-tui`. Removing it is a breaking change to the public API, but this is allowed per project policy.

## Reference Documentation
**Required:**
- Design: specs/tui-observation-mode/design.md (Section 4.3)

**Additional References:**
- specs/tui-observation-mode/context.md (state management patterns)
- specs/tui-observation-mode/plan.md (Step 3)

**Note:** You MUST read the design document before beginning implementation.

## Technical Requirements
1. Remove `LoopMode` enum definition from `state.rs` (lines 7-12)
2. Remove `pub loop_mode: LoopMode` field from `TuiState` struct (line 33)
3. Remove `loop_mode: LoopMode::Auto` from `new()` initialization (line 62)
4. Remove `loop_mode` from `with_hat_map()` initialization if present
5. Remove `LoopMode` from public exports in `lib.rs` (line 24)

## Dependencies
- Task 02 (command variants removed) - ensures no code tries to set LoopMode::Paused

## Implementation Approach
1. Read `crates/ralph-tui/src/state.rs` and `crates/ralph-tui/src/lib.rs`
2. Remove the enum definition from state.rs
3. Remove the field from TuiState struct
4. Remove initialization in constructors
5. Update lib.rs exports
6. Note: This WILL cause compile errors in app.rs and header.rs (fixed in subsequent tasks)
7. Run `cargo check -p ralph-tui` to see expected errors (should only be in app.rs, header.rs)

## Acceptance Criteria

1. **LoopMode enum removed**
   - Given state.rs is modified
   - When searching for "LoopMode" in state.rs
   - Then no matches are found

2. **TuiState field removed**
   - Given TuiState struct is modified
   - When inspecting its fields
   - Then no `loop_mode` field exists

3. **Public export removed**
   - Given lib.rs is modified
   - When checking exports
   - Then only `TuiState` is exported from state module (not `LoopMode`)

4. **Expected compile errors**
   - Given changes are complete
   - When running `cargo check -p ralph-tui`
   - Then errors appear only in app.rs and header.rs (referencing removed LoopMode)

## Metadata
- **Complexity**: Low
- **Labels**: tui, state-management, breaking-change
- **Required Skills**: Rust, module exports
