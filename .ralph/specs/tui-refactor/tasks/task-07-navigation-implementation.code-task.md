---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Implement Navigation

## Description
Ensure navigation methods are fully functional and integrate with the header/footer displays. This task validates that the navigation logic from Task 3 properly updates all dependent displays.

## Background
Navigation allows users to move between iterations using ←/→ arrow keys (or h/l vim keys). When navigating backwards from the latest iteration, the mode changes to REVIEW and a new iteration alert should appear in the footer if new iterations arrive. Navigating forward to the latest iteration restores LIVE mode.

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: User Interactions > Navigation)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- `ralph-tui/src/state.rs` — Navigation methods (from Task 3)

**Note:** You MUST read the design document before beginning.

## Technical Requirements
1. Verify `navigate_next()` and `navigate_prev()` work correctly (implemented in Task 3)
2. Ensure `following_latest` flag updates correctly on navigation
3. Implement `new_iteration_alert` tracking:
   - Set when new iteration arrives while `following_latest = false`
   - Clear when `following_latest` becomes true
4. Integration test: navigation → header update → footer update

## Dependencies
- Task 3: TuiState refactor (navigation methods already implemented)
- Task 5: Header widget update (displays mode indicator)
- Task 6: Footer widget update (displays new iteration alert)

## Implementation Approach
1. **GREEN**: Methods already implemented in Task 3
2. **REFACTOR**: Ensure bounds checking is robust, add alert tracking
3. **TEST**: Write integration tests verifying navigation updates all displays

## Acceptance Criteria

1. **Navigate Methods Work**
   - Given TuiState with 3 iterations
   - When `navigate_next()` and `navigate_prev()` are called
   - Then `current_view` updates correctly within bounds

2. **Following Latest Updates**
   - Given navigation from latest to previous
   - When `navigate_prev()` is called
   - Then `following_latest` becomes false

3. **Following Latest Restored**
   - Given `following_latest = false`
   - When `navigate_next()` reaches last iteration
   - Then `following_latest` becomes true

4. **New Iteration Alert Set**
   - Given `following_latest = false` and new iteration arrives
   - When `start_new_iteration()` is called
   - Then `new_iteration_alert` is set to the new iteration number

5. **Alert Cleared On Follow**
   - Given `new_iteration_alert = Some(5)`
   - When navigation restores `following_latest = true`
   - Then `new_iteration_alert` is cleared to None

6. **Integration Tests Pass**
   - Given the implementation is complete
   - When running navigation integration tests
   - Then all tests pass

## Metadata
- **Complexity**: Low
- **Labels**: interaction, navigation, tui
- **Required Skills**: Rust, state management, integration testing
