---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Create ContentPane Widget

## Description
Create the `ContentPane` widget that renders the current iteration's content with proper styling, scroll support, and search highlight capability. This replaces the VT100 terminal widget with a simpler line-based renderer.

## Background
The current TUI uses `tui-term` to render a VT100 terminal emulation screen. The new `ContentPane` widget will directly render the `Vec<Line>` from the current `IterationBuffer`, respecting scroll offset and optionally highlighting search matches. This is a significant simplification.

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: Components > ContentPane)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- `ralph-tui/src/widgets/header.rs:29-104` — Widget rendering pattern
- `ralph-tui/src/scroll.rs` — ScrollManager (may reuse concepts)

**Note:** You MUST read the design document before beginning implementation.

## Technical Requirements
1. Create `ralph-tui/src/widgets/content.rs`
2. Export from `ralph-tui/src/widgets/mod.rs`
3. Implement `ContentPane` as a ratatui Widget
4. Render lines from IterationBuffer using `visible_lines(viewport_height)`
5. Handle empty buffer gracefully (no panic, show empty area)
6. Support optional search highlight (mark spans matching search query)
7. Use existing styling patterns from other widgets

## Dependencies
- Task 1: IterationBuffer (ContentPane renders its lines)
- Task 3: TuiState (ContentPane gets buffer from TuiState)

## Implementation Approach
1. **RED**: Write failing widget render tests
2. **GREEN**: Implement ContentPane widget
3. **REFACTOR**: Optimize line slicing, ensure clean rendering

## Acceptance Criteria

1. **Renders Lines**
   - Given a buffer with 3 lines
   - When ContentPane renders with viewport height >= 3
   - Then all 3 lines are visible in the output

2. **Respects Scroll Offset**
   - Given a buffer with 10 lines and scroll_offset 5
   - When ContentPane renders with viewport height 5
   - Then lines 5-9 are shown

3. **Search Highlight**
   - Given a buffer with lines containing "foo"
   - When ContentPane renders with search query "foo"
   - Then "foo" spans are highlighted (different style)

4. **Empty Buffer Handling**
   - Given an empty IterationBuffer
   - When ContentPane renders
   - Then no panic occurs and empty area is shown

5. **Widget Integration**
   - Given ContentPane widget
   - When used with ratatui's render method
   - Then it correctly fills the provided Rect area

6. **Unit Tests Pass**
   - Given the implementation is complete
   - When running `cargo test -p ralph-tui content_pane`
   - Then all tests pass

## Metadata
- **Complexity**: Medium
- **Labels**: widgets, rendering, tui
- **Required Skills**: Rust, ratatui Widget trait, line rendering
