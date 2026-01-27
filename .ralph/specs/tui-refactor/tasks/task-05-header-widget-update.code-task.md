---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Update Header Widget

## Description
Update the header widget to display iteration position (`[iter N/M]`) and mode indicator (`[LIVE]` or `[REVIEW]`). The header should clearly communicate which iteration is being viewed and whether new content will auto-scroll into view.

## Background
The current header shows iteration number, elapsed time, and hat information. The updated header adds:
- Iteration position format: `[iter N/M]` showing current/total
- Mode indicator: `[LIVE]` when following latest, `[REVIEW]` when viewing history

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: Components > Header Widget)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- `ralph-tui/src/widgets/header.rs:29-104` — Current header implementation

**Note:** You MUST read the design document and current header code before beginning.

## Technical Requirements
1. Modify `ralph-tui/src/widgets/header.rs`
2. Add iteration position display: `[iter N/M]` format
3. Add mode indicator: `[LIVE]` (green) or `[REVIEW]` (yellow)
4. Read `current_view`, `total_iterations()`, and `following_latest` from TuiState
5. Maintain existing displays: elapsed time, hat emoji/name
6. Use progressive disclosure for width constraints

## Dependencies
- Task 3: TuiState refactor (provides `current_view`, `following_latest`, `total_iterations()`)

## Implementation Approach
1. **RED**: Add failing tests for new format elements
2. **GREEN**: Update render function with new displays
3. **REFACTOR**: Adjust progressive disclosure for width constraints

## Acceptance Criteria

1. **Iteration Position Format**
   - Given current_view = 3 and total_iterations = 5
   - When header renders
   - Then output contains `[iter 3/5]`

2. **Mode Live**
   - Given `following_latest = true`
   - When header renders
   - Then output contains `[LIVE]` (with green styling)

3. **Mode Review**
   - Given `following_latest = false`
   - When header renders
   - Then output contains `[REVIEW]` (with yellow styling)

4. **Hat Display Preserved**
   - Given hat = "Builder" with emoji "🔨"
   - When header renders
   - Then output contains "🔨 Builder"

5. **Elapsed Time Preserved**
   - Given 5 minutes elapsed
   - When header renders
   - Then output contains "05:00" format

6. **Unit Tests Pass**
   - Given the implementation is complete
   - When running `cargo test -p ralph-tui header`
   - Then all tests pass

## Metadata
- **Complexity**: Low
- **Labels**: widgets, header, tui
- **Required Skills**: Rust, ratatui styling, progressive disclosure
