---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Create IterationBuffer Data Structure

## Description
Create the `IterationBuffer` struct that stores content for a single iteration. This is the foundation data structure that will hold the styled output lines for each Ralph iteration, enabling pagination and scroll within each iteration's content.

## Background
The TUI refactor transforms ralph-tui from an interactive VT100 terminal emulator to a read-only observation dashboard. Each iteration needs its own content buffer with independent scroll state. The `IterationBuffer` is the fundamental unit that stores and manages content for one iteration.

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: Data Structures > IterationBuffer)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- `ralph-tui/src/state.rs:8-39` — Current TuiState structure (reference for integration)

**Note:** You MUST read the design document before beginning implementation.

## Technical Requirements
1. Create `IterationBuffer` struct in `ralph-tui/src/state.rs`
2. Store iteration number (1-indexed)
3. Store lines as `Vec<Line<'static>>` (ratatui Line type)
4. Track scroll offset (usize)
5. Implement line appending
6. Implement scroll methods: `scroll_up()`, `scroll_down()`, `scroll_top()`, `scroll_bottom()`
7. Implement `visible_lines(viewport_height: usize)` to return slice for rendering
8. All methods must be safe for empty buffers

## Dependencies
- None (this is the foundation task)

## Implementation Approach
1. **RED**: Write failing tests for IterationBuffer creation and line management
2. **GREEN**: Implement the IterationBuffer struct with minimal code to pass tests
3. **REFACTOR**: Clean up helper methods, ensure bounds checking is robust

## Acceptance Criteria

1. **Buffer Creation**
   - Given no existing buffer
   - When `IterationBuffer::new(1)` is called
   - Then a buffer with iteration number 1, empty lines, and scroll_offset 0 is created

2. **Line Appending**
   - Given an IterationBuffer
   - When `append_line()` is called with a Line
   - Then the line is added and order is preserved

3. **Line Count**
   - Given a buffer with 10 lines
   - When `line_count()` is called
   - Then 10 is returned

4. **Visible Lines Without Scroll**
   - Given a buffer with 10 lines and viewport height 5
   - When `visible_lines(5)` is called with scroll_offset 0
   - Then lines 0-4 are returned

5. **Visible Lines With Scroll**
   - Given a buffer with 10 lines, viewport height 5, scroll_offset 3
   - When `visible_lines(5)` is called
   - Then lines 3-7 are returned

6. **Scroll Down**
   - Given a buffer with content
   - When `scroll_down()` is called
   - Then scroll_offset increases by 1

7. **Scroll Up**
   - Given a buffer with scroll_offset > 0
   - When `scroll_up()` is called
   - Then scroll_offset decreases by 1

8. **Scroll Bounds At Start**
   - Given a buffer with scroll_offset 0
   - When `scroll_up()` is called
   - Then scroll_offset remains 0 (no underflow)

9. **Scroll Bounds At End**
   - Given a buffer with 10 lines and scroll_offset at max
   - When `scroll_down()` is called
   - Then scroll_offset remains capped (no overflow)

10. **Scroll Top**
    - Given a buffer with scroll_offset > 0
    - When `scroll_top()` is called
    - Then scroll_offset becomes 0

11. **Scroll Bottom**
    - Given a buffer with content
    - When `scroll_bottom(viewport_height)` is called
    - Then scroll_offset is set so last lines are visible

12. **Unit Tests Pass**
    - Given the implementation is complete
    - When running `cargo test -p ralph-tui iteration_buffer`
    - Then all tests pass

## Metadata
- **Complexity**: Medium
- **Labels**: foundation, data-structure, tui
- **Required Skills**: Rust, ratatui Line type, basic data structures
