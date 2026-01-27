---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Simplify Input Handling

## Description
Rewrite the input handler to be a simple key-to-action mapper without the previous prefix key complexity. All keys map directly to actions since there's no need for interactive input forwarding.

## Background
The previous input handler supported an "interactive mode" where certain key combinations would prefix input to the underlying process. Since the TUI is now read-only observation mode, all keys can map directly to TUI actions without any mode-switching complexity.

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: User Interactions > Input Handling)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- `ralph-tui/src/input.rs` — Current input handler

**Note:** You MUST read the design document and current input handler before beginning.

## Technical Requirements
1. Rewrite `ralph-tui/src/input.rs` with simple key mappings
2. Define `Action` enum with all TUI actions:
   - `Quit`, `NextIteration`, `PrevIteration`
   - `ScrollDown`, `ScrollUp`, `ScrollTop`, `ScrollBottom`
   - `StartSearch`, `SearchNext`, `SearchPrev`
   - `ShowHelp`, `DismissHelp`
   - `None` (unknown key)
3. Implement `map_key(KeyEvent) -> Action` function
4. Support both arrow keys and vim-style navigation
5. Remove old InputRouter code that handled prefix keys

## Dependencies
- Task 7: Navigation implementation (actions trigger navigation)
- Task 8: Scroll implementation (actions trigger scroll)
- Task 9: Search implementation (actions trigger search)

## Implementation Approach
1. **RED**: Write comprehensive input tests for all key mappings
2. **GREEN**: Rewrite input handler with simple match statement
3. **REFACTOR**: Remove old InputRouter code, clean up unused patterns

## Acceptance Criteria

1. **q Quits**
   - Given key `KeyCode::Char('q')`
   - When `map_key()` is called
   - Then `Action::Quit` is returned

2. **Right Arrow Next Iteration**
   - Given key `KeyCode::Right`
   - When `map_key()` is called
   - Then `Action::NextIteration` is returned

3. **Left Arrow Prev Iteration**
   - Given key `KeyCode::Left`
   - When `map_key()` is called
   - Then `Action::PrevIteration` is returned

4. **j Scroll Down**
   - Given key `KeyCode::Char('j')`
   - When `map_key()` is called
   - Then `Action::ScrollDown` is returned

5. **k Scroll Up**
   - Given key `KeyCode::Char('k')`
   - When `map_key()` is called
   - Then `Action::ScrollUp` is returned

6. **g Scroll Top**
   - Given key `KeyCode::Char('g')`
   - When `map_key()` is called
   - Then `Action::ScrollTop` is returned

7. **G Scroll Bottom**
   - Given key `KeyCode::Char('G')`
   - When `map_key()` is called
   - Then `Action::ScrollBottom` is returned

8. **/ Start Search**
   - Given key `KeyCode::Char('/')`
   - When `map_key()` is called
   - Then `Action::StartSearch` is returned

9. **n Search Next**
   - Given key `KeyCode::Char('n')`
   - When `map_key()` is called
   - Then `Action::SearchNext` is returned

10. **N Search Prev**
    - Given key `KeyCode::Char('N')`
    - When `map_key()` is called
    - Then `Action::SearchPrev` is returned

11. **? Show Help**
    - Given key `KeyCode::Char('?')`
    - When `map_key()` is called
    - Then `Action::ShowHelp` is returned

12. **Esc Dismiss Help**
    - Given key `KeyCode::Esc`
    - When `map_key()` is called
    - Then `Action::DismissHelp` is returned

13. **Vim l Next Iteration**
    - Given key `KeyCode::Char('l')`
    - When `map_key()` is called
    - Then `Action::NextIteration` is returned

14. **Vim h Prev Iteration**
    - Given key `KeyCode::Char('h')`
    - When `map_key()` is called
    - Then `Action::PrevIteration` is returned

15. **Unknown Key Returns None**
    - Given key `KeyCode::Char('x')` (unmapped)
    - When `map_key()` is called
    - Then `Action::None` is returned

16. **Unit Tests Pass**
    - Given the implementation is complete
    - When running `cargo test -p ralph-tui input`
    - Then all tests pass

## Metadata
- **Complexity**: Medium
- **Labels**: interaction, input, tui
- **Required Skills**: Rust, crossterm KeyEvent, match patterns
