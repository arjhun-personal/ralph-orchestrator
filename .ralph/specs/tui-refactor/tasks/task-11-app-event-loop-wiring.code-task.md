---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Wire App Event Loop

## Description
Refactor the app event loop to use the new components (TuiStreamHandler, ContentPane, simplified input) and remove PTY handling code. The event loop should handle keyboard input, stream events, and render updates.

## Background
The current app event loop manages PTY communication, terminal rendering, and keyboard input with prefix key handling. The refactored loop is simpler:
- Receives events from TuiStreamHandler (new content) and event observer (state updates)
- Handles keyboard input via simple action mapping
- Renders using header, ContentPane, and footer widgets

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: Architecture Overview)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- `ralph-tui/src/app.rs:132-323` — Current event loop pattern

**Note:** You MUST read the design document and current app.rs before beginning.

## Technical Requirements
1. Refactor `ralph-tui/src/app.rs`
2. Remove PTY output handling task (no more VT100 parsing)
3. Wire TuiStreamHandler output to TuiState
4. Connect keyboard events to action dispatch
5. Connect actions to state mutations
6. Update rendering to use ContentPane instead of terminal widget
7. Handle quit action and graceful shutdown

## Dependencies
- Task 2: TuiStreamHandler (provides content)
- Task 3: TuiState refactor (manages state)
- Task 4: ContentPane widget (renders content)
- Task 5: Header widget update (renders header)
- Task 6: Footer widget update (renders footer)
- Task 10: Input handling simplification (action mapping)

## Implementation Approach
1. **RED**: Write integration tests for event flow
2. **GREEN**: Refactor app.rs to use new components
3. **REFACTOR**: Remove PTY handling code, clean up unused imports

## Acceptance Criteria

1. **Events Reach State**
   - Given TuiStreamHandler receives text
   - When event is processed
   - Then current IterationBuffer is updated

2. **Keyboard Triggers Actions**
   - Given keyboard input 'j'
   - When input is processed
   - Then scroll_down() is called on current buffer

3. **Render Uses ContentPane**
   - Given app event loop
   - When render is called
   - Then ContentPane renders the current iteration's content

4. **Header/Footer Integrated**
   - Given app event loop
   - When render is called
   - Then header and footer render with correct state

5. **Quit Exits Cleanly**
   - Given keyboard input 'q'
   - When input is processed
   - Then event loop terminates and terminal is restored

6. **No PTY Code**
   - Given refactored app.rs
   - When examining the code
   - Then no PTY or VT100 related code remains

7. **Integration Tests Pass**
   - Given the implementation is complete
   - When running `cargo test -p ralph-tui app`
   - Then all tests pass

## Metadata
- **Complexity**: High
- **Labels**: integration, event-loop, tui
- **Required Skills**: Rust, async/tokio, ratatui rendering, event handling
