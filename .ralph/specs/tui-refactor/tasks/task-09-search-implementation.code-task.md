---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Implement Search

## Description
Implement search functionality that finds text matches in the current iteration's content, highlights matches, and allows navigation between matches using n/N keys.

## Background
Search allows users to find specific content within the current iteration. The `/` key activates search mode, the user types a query, matches are highlighted in the ContentPane, and n/N navigate between matches. Search is case-insensitive and cycles through matches.

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: User Interactions > Search)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- `ralph-tui/src/scroll.rs` — ScrollManager (may reuse concepts)

**Note:** You MUST read the design document before beginning.

## Technical Requirements
1. Add `SearchState` struct to `ralph-tui/src/state.rs`:
   - `query: Option<String>` — current search query
   - `matches: Vec<(usize, usize)>` — (line_index, char_offset) pairs
   - `current_match: usize` — index into matches vector
   - `search_mode: bool` — whether search input is active
2. Implement `search(query: &str)` — finds all matches in current iteration
3. Implement `next_match()` — moves to next match, cycles
4. Implement `prev_match()` — moves to previous match, cycles
5. Implement `clear_search()` — clears search state
6. Search should be case-insensitive
7. When jumping to a match, scroll_offset should update to show the match

## Dependencies
- Task 1: IterationBuffer (search within its lines)
- Task 4: ContentPane widget (highlights matches)
- Task 6: Footer widget update (shows search query)

## Implementation Approach
1. **RED**: Write failing search tests
2. **GREEN**: Implement SearchState and methods
3. **REFACTOR**: Optimize search algorithm if needed

## Acceptance Criteria

1. **Search Finds Matches**
   - Given current iteration with "error" in 3 lines
   - When `search("error")` is called
   - Then `matches.len() == 3` (or more if multiple per line)

2. **Search Case Insensitive**
   - Given current iteration with "Error" and "error"
   - When `search("error")` is called
   - Then both are found

3. **Next Match Cycles**
   - Given 3 matches and `current_match = 2`
   - When `next_match()` is called
   - Then `current_match` becomes 0 (cycles back)

4. **Prev Match Cycles**
   - Given 3 matches and `current_match = 0`
   - When `prev_match()` is called
   - Then `current_match` becomes 2 (cycles back)

5. **Search Jumps To Match**
   - Given match at line 50
   - When navigating to that match
   - Then `scroll_offset` is updated so line 50 is visible

6. **Clear Search**
   - Given active search
   - When `clear_search()` is called
   - Then `query = None`, `matches` cleared, `search_mode = false`

7. **Unit Tests Pass**
   - Given the implementation is complete
   - When running `cargo test -p ralph-tui search`
   - Then all tests pass

## Metadata
- **Complexity**: Medium
- **Labels**: interaction, search, tui
- **Required Skills**: Rust, string searching, state management
