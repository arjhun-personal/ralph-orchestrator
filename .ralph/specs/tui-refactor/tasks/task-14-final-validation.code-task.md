---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Final Validation

## Description
Execute the full E2E test scenario manually, run all automated tests, verify output parity, and record a demo for PR documentation.

## Background
This is the final validation gate before the refactor is complete. It ensures all components work together correctly, output matches the non-TUI mode, and provides evidence of completion.

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md
- specs/tui-refactor/plan.md (Section: E2E Test Scenario, Success Criteria)

**Additional References:**
- CLAUDE.md (PR demo instructions)
- specs/tui-refactor/context.md

**Note:** Follow the E2E test scenario checklist exactly as written in plan.md.

## Technical Requirements
1. Execute E2E test scenario from plan.md
2. Run full test suite: `cargo test`
3. Run Clippy: `cargo clippy`
4. Run build: `cargo build`
5. Verify output parity with non-TUI mode
6. Optional: Record asciinema demo for PR

## Dependencies
- All previous tasks (1-13) must be complete

## Implementation Approach
1. **VERIFY**: Execute E2E scenario checklist
2. **VERIFY**: All automated tests pass
3. **VERIFY**: Clean Clippy and build
4. **DOCUMENT**: Record demo if requested

## Acceptance Criteria

1. **E2E Scenario Complete**
   - Given E2E scenario checklist from plan.md
   - When executing all steps
   - Then all checkboxes pass

2. **Unit Tests Pass**
   - Given implementation is complete
   - When running `cargo test`
   - Then all ~50 unit tests pass

3. **Integration Tests Pass**
   - Given implementation is complete
   - When running integration tests
   - Then all 6 integration tests pass

4. **Clean Build**
   - Given implementation is complete
   - When running `cargo build`
   - Then no warnings (beyond allowed)

5. **Clean Clippy**
   - Given implementation is complete
   - When running `cargo clippy`
   - Then no warnings

6. **Smoke Tests Pass**
   - Given implementation is complete
   - When running `cargo test -p ralph-core smoke_runner`
   - Then smoke tests pass

7. **Output Parity Verified**
   - Given same content through TUI and non-TUI modes
   - When comparing output
   - Then formatting is equivalent

8. **TUI Responsive**
   - Given running TUI session
   - When using all keyboard shortcuts
   - Then all respond correctly:
     - ←/→ or h/l for iteration navigation
     - j/k for scroll
     - g/G for jump
     - / for search, n/N for match navigation
     - ? for help, Esc to dismiss
     - q to quit

9. **Demo Recorded (Optional)**
   - Given completion of all validation
   - When requested
   - Then asciinema demo is recorded per CLAUDE.md instructions

## E2E Checklist (from plan.md)

### Initial Display
- [ ] Header shows `[iter 1/1]` and `[LIVE]`
- [ ] Hat emoji and name appear
- [ ] Elapsed time ticking
- [ ] Footer shows activity indicator `◉`

### Content Display
- [ ] Text output appears in content area
- [ ] Tool calls show with `⚙️` icon
- [ ] Formatting matches non-TUI output

### Iteration Navigation
- [ ] Press `←` - header shows `[iter 1/N]` and `[REVIEW]`
- [ ] Content shows iteration 1 output
- [ ] Footer shows "▶ New: iter N" alert
- [ ] Press `→` - returns to latest, `[LIVE]`
- [ ] Alert clears

### Scroll
- [ ] Press `j` - content scrolls down
- [ ] Press `k` - content scrolls up
- [ ] Press `G` - jumps to bottom
- [ ] Press `g` - jumps to top

### Search
- [ ] Press `/` - footer shows search input
- [ ] Type query - matches highlighted
- [ ] Press `n` - jumps to next match
- [ ] Press `N` - jumps to previous match
- [ ] Press `Esc` - clears search

### Help and Exit
- [ ] Press `?` - help overlay appears
- [ ] Press `Esc` - help dismisses
- [ ] Press `q` - TUI exits cleanly
- [ ] Terminal restored properly

## Metadata
- **Complexity**: Medium
- **Labels**: validation, e2e, tui
- **Required Skills**: Manual testing, test execution, demo recording
