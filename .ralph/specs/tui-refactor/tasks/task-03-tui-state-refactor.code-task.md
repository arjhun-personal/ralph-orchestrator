---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Refactor TuiState for Iteration Management

## Description
Refactor `TuiState` to manage multiple `IterationBuffer` instances and support navigation between iterations. This transforms the state from tracking PTY terminal output to tracking discrete iteration content buffers.

## Background
Currently, `TuiState` tracks a single terminal screen state via the PTY/VT100 emulation. The refactored version will maintain a vector of `IterationBuffer` instances, track which iteration is being viewed, and support "following latest" mode vs "review" mode for historical iteration viewing.

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: Data Structures > TuiState)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- `ralph-tui/src/state.rs:8-39` — Current TuiState structure

**Note:** You MUST read the design document before beginning implementation.

## Technical Requirements
1. Add `iterations: Vec<IterationBuffer>` to TuiState
2. Add `current_view: usize` — which iteration is being displayed
3. Add `following_latest: bool` — whether to auto-follow new iterations
4. Implement `start_new_iteration()` — creates new IterationBuffer
5. Implement `current_iteration() -> &IterationBuffer` — returns buffer for current_view
6. Implement `current_iteration_mut() -> &mut IterationBuffer` — mutable version
7. Implement `navigate_next()` / `navigate_prev()` — changes current_view
8. Implement `total_iterations() -> usize`
9. Remove deprecated fields: `pending_hat`, `in_scroll_mode` (if they exist and are unused)

## Dependencies
- Task 1: IterationBuffer (TuiState manages Vec<IterationBuffer>)

## Implementation Approach
1. **RED**: Write failing tests for iteration management
2. **GREEN**: Refactor TuiState with new fields and methods
3. **REFACTOR**: Remove deprecated fields, ensure clean API

## Acceptance Criteria

1. **Start New Iteration**
   - Given TuiState with 0 iterations
   - When `start_new_iteration()` is called
   - Then `iterations.len() == 1` and new IterationBuffer exists

2. **Current Iteration Returns Correct Buffer**
   - Given TuiState with 3 iterations and `current_view = 1`
   - When `current_iteration()` is called
   - Then the buffer at index 1 is returned

3. **Navigate Next**
   - Given TuiState with `current_view = 1` and 3 iterations
   - When `navigate_next()` is called
   - Then `current_view == 2`

4. **Navigate Prev**
   - Given TuiState with `current_view = 2`
   - When `navigate_prev()` is called
   - Then `current_view == 1`

5. **Navigate Bounds - Cannot Exceed**
   - Given TuiState with `current_view = 2` and 3 iterations (max index 2)
   - When `navigate_next()` is called
   - Then `current_view` stays at 2

6. **Navigate Bounds - Cannot Go Below Zero**
   - Given TuiState with `current_view = 0`
   - When `navigate_prev()` is called
   - Then `current_view` stays at 0

7. **Following Latest Initially True**
   - Given new TuiState
   - When created
   - Then `following_latest == true`

8. **Following Latest Becomes False On Back Navigation**
   - Given TuiState with `following_latest = true` and `current_view = 2`
   - When `navigate_prev()` is called
   - Then `following_latest == false`

9. **Following Latest Restored At Latest**
   - Given TuiState with `following_latest = false`
   - When `navigate_next()` reaches the last iteration
   - Then `following_latest == true`

10. **Total Iterations Reports Count**
    - Given TuiState with 3 iterations
    - When `total_iterations()` is called
    - Then 3 is returned

11. **Unit Tests Pass**
    - Given the implementation is complete
    - When running `cargo test -p ralph-tui tui_state`
    - Then all tests pass

## Metadata
- **Complexity**: Medium
- **Labels**: foundation, state-management, tui
- **Required Skills**: Rust, state management patterns, Vec manipulation
