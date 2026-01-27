---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Implement Scroll

## Description
Ensure scroll methods are fully functional with proper viewport height awareness. This task validates that the scroll logic from Task 1 integrates correctly with the ContentPane rendering.

## Background
Scroll allows users to navigate within an iteration's content using j/k keys for single-line scroll and g/G for jump to top/bottom. Each iteration maintains its own independent scroll state, so switching iterations preserves each iteration's scroll position.

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: User Interactions > Scroll)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- `ralph-tui/src/state.rs` — IterationBuffer scroll methods (from Task 1)
- `ralph-tui/src/scroll.rs` — ScrollManager (may be deprecated or merged)

**Note:** You MUST read the design document before beginning.

## Technical Requirements
1. Verify scroll methods in IterationBuffer work correctly (implemented in Task 1)
2. Add viewport height parameter to scroll calculations for `scroll_bottom()`
3. Ensure per-iteration scroll independence (switching iterations preserves scroll)
4. Connect scroll state to ContentPane rendering

## Dependencies
- Task 1: IterationBuffer (scroll methods already implemented)
- Task 4: ContentPane widget (uses scroll_offset for rendering)

## Implementation Approach
1. **GREEN**: Methods already implemented in Task 1
2. **REFACTOR**: Add viewport height awareness to `scroll_bottom()`
3. **TEST**: Write integration tests verifying scroll → ContentPane rendering

## Acceptance Criteria

1. **Scroll Down Works**
   - Given IterationBuffer with content
   - When `scroll_down()` is called
   - Then `scroll_offset` increases by 1

2. **Scroll Up Works**
   - Given IterationBuffer with `scroll_offset > 0`
   - When `scroll_up()` is called
   - Then `scroll_offset` decreases by 1

3. **Scroll Top Jumps To Start**
   - Given IterationBuffer with `scroll_offset > 0`
   - When `scroll_top()` is called
   - Then `scroll_offset` becomes 0

4. **Scroll Bottom Jumps To End**
   - Given IterationBuffer with many lines and viewport height 10
   - When `scroll_bottom(10)` is called
   - Then `scroll_offset` is set so last 10 lines are visible

5. **Per-Iteration Scroll Independence**
   - Given iteration 1 with scroll_offset 5 and iteration 2 with scroll_offset 0
   - When switching between iterations
   - Then each iteration's scroll_offset is preserved

6. **ContentPane Uses Scroll**
   - Given IterationBuffer with scroll_offset 5
   - When ContentPane renders
   - Then lines starting from offset 5 are displayed

7. **Integration Tests Pass**
   - Given the implementation is complete
   - When running scroll integration tests
   - Then all tests pass

## Metadata
- **Complexity**: Low
- **Labels**: interaction, scroll, tui
- **Required Skills**: Rust, viewport calculations, state management
