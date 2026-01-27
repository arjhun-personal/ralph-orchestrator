---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Update Footer Widget

## Description
Update the footer widget to display new iteration alerts, search query in search mode, and maintain existing activity indicator. The footer communicates real-time status and provides search feedback.

## Background
The current footer shows activity indicator and last event. The updated footer adds:
- New iteration alert: `▶ New: iter N` when viewing history and new iteration arrives
- Search query display: `/query` when in search mode
- Alert should clear when user navigates to the new iteration

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: Components > Footer Widget)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- `ralph-tui/src/widgets/footer.rs` — Current footer implementation

**Note:** You MUST read the design document and current footer code before beginning.

## Technical Requirements
1. Modify `ralph-tui/src/widgets/footer.rs`
2. Add new iteration alert: `▶ New: iter N`
3. Add search query display: `/query` when search_mode is active
4. Track `new_iteration_alert: Option<usize>` in TuiState (iteration number to alert about)
5. Clear alert when `following_latest` becomes true
6. Maintain existing activity indicator (`◉`/`◯`/`■`)

## Dependencies
- Task 3: TuiState refactor (provides state for alerts)
- Task 9: Search implementation (provides search query)

## Implementation Approach
1. **RED**: Add failing tests for alert and search display
2. **GREEN**: Update render function with new displays
3. **REFACTOR**: Layout adjustments for all elements

## Acceptance Criteria

1. **New Iteration Alert Shows**
   - Given `new_iteration_alert = Some(5)` and `following_latest = false`
   - When footer renders
   - Then output contains "▶ New: iter 5"

2. **No Alert When Following**
   - Given `following_latest = true`
   - When footer renders
   - Then no alert is shown (even if new_iteration_alert has a value)

3. **Last Event Shown**
   - Given `last_event = Some("build.done")`
   - When footer renders
   - Then output contains "build.done"

4. **Activity Indicator Active**
   - Given activity is ongoing
   - When footer renders
   - Then output contains `◉` (active indicator)

5. **Search Query Shown**
   - Given `search_mode = true` and search query = "test"
   - When footer renders
   - Then output contains "/test"

6. **Unit Tests Pass**
   - Given the implementation is complete
   - When running `cargo test -p ralph-tui footer`
   - Then all tests pass

## Metadata
- **Complexity**: Low
- **Labels**: widgets, footer, tui
- **Required Skills**: Rust, ratatui styling, layout management
