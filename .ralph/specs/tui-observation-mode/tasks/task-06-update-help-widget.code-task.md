---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Update Help Widget

## Description
Update the help widget to remove documentation for pause/skip/abort commands and add comprehensive documentation for scroll mode and search functionality.

## Background
The help screen (shown via `Ctrl+a ?`) currently documents all prefix commands including p/n/a for execution control. These must be removed, and scroll/search documentation should be enhanced (R4.2 requirement).

## Reference Documentation
**Required:**
- Design: specs/tui-observation-mode/design.md (Section 4.5 - Help Widget)

**Additional References:**
- specs/tui-observation-mode/context.md
- specs/tui-observation-mode/plan.md (Step 6)

**Note:** You MUST read the design document before beginning implementation.

## Technical Requirements
1. Remove pause line: `"  p", "  Pause/resume loop"` (lines 32-35)
2. Remove skip line: `"  n", "  Skip to next iteration"` (lines 36-39)
3. Remove abort line: `"  a", "  Abort loop"` (lines 40-43)
4. Add "Scroll Mode:" section with vim-style navigation docs
5. Add search documentation (`/`, `?`, `n/N`)

## Dependencies
- Task 05 (header updated) - TUI should fully compile before this

## Implementation Approach
1. Read `crates/ralph-tui/src/widgets/help.rs`
2. Locate the help text vector
3. Remove the three lines for p, n, a commands
4. Add scroll mode section with navigation keys (j/k, gg/G, Ctrl+u/d)
5. Add search section (/, ?, n/N)
6. Verify with `cargo build` and visual inspection if possible

## Acceptance Criteria

1. **Pause documentation removed**
   - Given help.rs is modified
   - When searching for "Pause" or "pause"
   - Then no help text matches are found

2. **Skip documentation removed**
   - Given help.rs is modified
   - When searching for "Skip" or "skip"
   - Then no help text matches are found

3. **Abort documentation removed**
   - Given help.rs is modified
   - When searching for "Abort" or "abort"
   - Then no help text matches are found

4. **Scroll mode documented**
   - Given help widget renders
   - When viewing help screen
   - Then "Scroll Mode:" section appears with j/k, gg/G, Ctrl+u/d documented

5. **Search documented**
   - Given help widget renders
   - When viewing help screen
   - Then search keys (/, ?, n/N) are documented

6. **Help compiles**
   - Given all changes are complete
   - When running `cargo build -p ralph-tui`
   - Then compilation succeeds

## Metadata
- **Complexity**: Low
- **Labels**: tui, widget, documentation
- **Required Skills**: Rust, ratatui text rendering
