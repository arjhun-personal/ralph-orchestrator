# TUI Refactor - Test Strategy & Implementation Plan

> Created by: 📋 Planner
> Source: `specs/tui-refactor/design/detailed-design.md`, `specs/tui-refactor/context.md`

## Test Plan

### Unit Tests

#### 1. IterationBuffer Tests (`ralph-tui/src/state.rs`)

| Test | Description | Inputs | Expected |
|------|-------------|--------|----------|
| `test_new_buffer_is_empty` | Fresh buffer has no lines | `IterationBuffer::new(1)` | `lines.len() == 0`, `scroll_offset == 0` |
| `test_append_line` | Lines accumulate correctly | Append 3 lines | `lines.len() == 3`, order preserved |
| `test_line_count` | Reports correct line count | Buffer with 10 lines | `line_count() == 10` |
| `test_visible_lines_no_scroll` | Returns first N lines | 10 lines, viewport 5, scroll 0 | Lines 0-4 |
| `test_visible_lines_with_scroll` | Respects scroll offset | 10 lines, viewport 5, scroll 3 | Lines 3-7 |
| `test_scroll_down_increments` | Scroll offset increases | `scroll_down()` on buffer | `scroll_offset += 1` |
| `test_scroll_up_decrements` | Scroll offset decreases | `scroll_up()` after scroll down | `scroll_offset -= 1` |
| `test_scroll_bounds_at_start` | Can't scroll above 0 | `scroll_up()` at top | `scroll_offset == 0` |
| `test_scroll_bounds_at_end` | Can't scroll past content | `scroll_down()` past end | Capped at max |
| `test_scroll_top` | Jumps to top | `scroll_top()` | `scroll_offset == 0` |
| `test_scroll_bottom` | Jumps to bottom | `scroll_bottom()` | `scroll_offset == max` |

#### 2. TuiStreamHandler Tests (`ralph-adapters/src/stream_handler.rs`)

| Test | Description | Inputs | Expected |
|------|-------------|--------|----------|
| `test_on_text_creates_line` | Text becomes ratatui Line | `on_text("hello")` | Line with "hello" content |
| `test_on_text_buffers_partial` | Partial text is buffered | `on_text("hel")` then `on_text("lo\n")` | Single "hello" line after newline |
| `test_on_tool_call_format` | Tool call formatted as icon + name | `on_tool_call("Read", ...)` | Line starts with "⚙️" |
| `test_on_tool_result_verbose` | Results shown in verbose mode | `verbose=true`, `on_tool_result(...)` | Result content in output |
| `test_on_tool_result_quiet` | Results hidden in quiet mode | `verbose=false`, `on_tool_result(...)` | Only checkmark shown |
| `test_on_error_red_style` | Errors styled red | `on_error("fail")` | Line has red foreground |
| `test_text_truncation` | Long lines truncated UTF-8 safe | 500+ char string | Truncated with "..." |

#### 3. Output Parity Tests (`ralph-adapters/src/stream_handler.rs`)

| Test | Description | Inputs | Expected |
|------|-------------|--------|----------|
| `test_parity_text_output` | Text formatting matches | Same text to both handlers | Content equivalent |
| `test_parity_tool_call` | Tool call format matches | Same tool call to both | Icon and name match |
| `test_parity_tool_result` | Tool result format matches | Same result to both | Format equivalent |
| `test_parity_error` | Error format matches | Same error to both | Both show red styled error |

#### 4. TuiState Tests (`ralph-tui/src/state.rs`)

| Test | Description | Inputs | Expected |
|------|-------------|--------|----------|
| `test_start_new_iteration` | Creates new buffer | `start_new_iteration()` | `iterations.len()` increases |
| `test_current_iteration` | Returns correct buffer | Multiple iterations | Returns buffer at `current_view` |
| `test_navigate_next` | Increments view | `navigate_next()` | `current_view += 1` |
| `test_navigate_prev` | Decrements view | `navigate_prev()` | `current_view -= 1` |
| `test_navigate_bounds` | Can't exceed bounds | Navigate past end | Stays at last |
| `test_following_latest_initial` | Starts following | New state | `following_latest == true` |
| `test_following_latest_after_nav` | Stops following on back | Navigate back | `following_latest == false` |
| `test_following_latest_restored` | Restored at latest | Navigate to end | `following_latest == true` |
| `test_total_iterations` | Reports count | 3 iterations | `total_iterations() == 3` |

#### 5. Input Router Tests (`ralph-tui/src/input.rs`)

| Test | Description | Inputs | Expected |
|------|-------------|--------|----------|
| `test_q_quits` | q → Quit | `KeyCode::Char('q')` | `Action::Quit` |
| `test_right_next_iter` | → → NextIteration | `KeyCode::Right` | `Action::NextIteration` |
| `test_left_prev_iter` | ← → PrevIteration | `KeyCode::Left` | `Action::PrevIteration` |
| `test_j_scroll_down` | j → ScrollDown | `KeyCode::Char('j')` | `Action::ScrollDown` |
| `test_k_scroll_up` | k → ScrollUp | `KeyCode::Char('k')` | `Action::ScrollUp` |
| `test_g_scroll_top` | g → ScrollTop | `KeyCode::Char('g')` | `Action::ScrollTop` |
| `test_G_scroll_bottom` | G → ScrollBottom | `KeyCode::Char('G')` | `Action::ScrollBottom` |
| `test_slash_search` | / → StartSearch | `KeyCode::Char('/')` | `Action::StartSearch` |
| `test_n_search_next` | n → SearchNext | `KeyCode::Char('n')` | `Action::SearchNext` |
| `test_N_search_prev` | N → SearchPrev | `KeyCode::Char('N')` | `Action::SearchPrev` |
| `test_question_help` | ? → ShowHelp | `KeyCode::Char('?')` | `Action::ShowHelp` |
| `test_esc_dismiss` | Esc → DismissHelp | `KeyCode::Esc` | `Action::DismissHelp` |
| `test_vim_l_next_iter` | l → NextIteration | `KeyCode::Char('l')` | `Action::NextIteration` |
| `test_vim_h_prev_iter` | h → PrevIteration | `KeyCode::Char('h')` | `Action::PrevIteration` |
| `test_unknown_none` | Unknown → None | `KeyCode::Char('x')` | `Action::None` |

#### 6. Header Widget Tests (`ralph-tui/src/widgets/header.rs`)

| Test | Description | Inputs | Expected |
|------|-------------|--------|----------|
| `test_iter_position_format` | Shows N/M format | iter 3 of 5 | Contains `[iter 3/5]` |
| `test_mode_live` | Shows LIVE when following | `following_latest=true` | Contains `[LIVE]` |
| `test_mode_review` | Shows REVIEW in history | `following_latest=false` | Contains `[REVIEW]` |
| `test_hat_display` | Shows current hat | hat = Builder | Contains "🔨 Builder" |
| `test_elapsed_time` | Shows elapsed time | 5 mins elapsed | Contains `05:00` |

#### 7. Footer Widget Tests (`ralph-tui/src/widgets/footer.rs`)

| Test | Description | Inputs | Expected |
|------|-------------|--------|----------|
| `test_new_iter_alert` | Shows alert when new | `new_iteration_alert=Some(5)` | Contains "▶ New: iter 5" |
| `test_no_alert_when_following` | No alert when following | `following_latest=true` | No alert shown |
| `test_last_event_shown` | Shows last event | `last_event=Some("build.done")` | Contains "build.done" |
| `test_activity_indicator_active` | Shows active dot | Activity ongoing | Shows `◉` |
| `test_search_query_shown` | Shows search in search mode | `search_mode=true`, query="test" | Contains "/test" |

#### 8. ContentPane Widget Tests (`ralph-tui/src/widgets/content.rs`)

| Test | Description | Inputs | Expected |
|------|-------------|--------|----------|
| `test_renders_lines` | Shows buffer content | Buffer with 3 lines | All 3 lines visible |
| `test_respects_scroll` | Scroll offset affects view | 10 lines, scroll 5 | Shows lines 5+ |
| `test_search_highlight` | Highlights search matches | Search "foo" with matches | Highlighted spans |
| `test_empty_buffer` | Handles empty buffer | No lines | Renders without panic |

#### 9. Search Tests (`ralph-tui/src/state.rs`)

| Test | Description | Inputs | Expected |
|------|-------------|--------|----------|
| `test_search_finds_matches` | Finds all occurrences | "error" in 3 lines | `matches.len() == 3` |
| `test_search_case_insensitive` | Case insensitive | "Error" vs "error" | Both match |
| `test_next_match_cycles` | Cycles through matches | 3 matches, next 4x | Returns to first |
| `test_prev_match_cycles` | Reverse cycle | 3 matches, prev from first | Goes to last |
| `test_search_jumps_to_match` | Scroll follows match | Match at line 50 | `scroll_offset` updated |
| `test_clear_search` | Clears search state | `clear_search()` | `search == None` |

### Integration Tests

#### 1. Output Parity Integration (`ralph-adapters/tests/`)

```rust
#[test]
fn test_full_session_parity() {
    // Feed recorded JSONL session to both handlers
    // Compare output structure (not exact strings due to styling differences)
}
```

| Test | Description |
|------|-------------|
| `test_full_session_parity` | Process recorded session through both handlers, verify equivalent structure |
| `test_multi_tool_sequence` | Multiple tool calls in sequence produce same order |
| `test_error_recovery_parity` | Error handling produces equivalent display |

#### 2. Iteration Navigation Integration (`ralph-tui/tests/`)

| Test | Description |
|------|-------------|
| `test_multiple_iterations` | Create 5 iterations, navigate all, verify content isolation |
| `test_content_isolation` | Content from iteration N doesn't appear in iteration M |
| `test_scroll_per_iteration` | Each iteration maintains independent scroll |

#### 3. App Event Loop Integration (`ralph-tui/tests/`)

| Test | Description |
|------|-------------|
| `test_app_receives_events` | Events reach TuiStreamHandler and update state |
| `test_app_keyboard_actions` | Keyboard input triggers correct state changes |

### E2E Test Scenario (Manual)

**Scenario: Full TUI Observation Session**

**Prerequisites:**
- Ralph built: `cargo build --release`
- Test config: `ralph.claude.yml` with simple prompt capability

**Steps:**

1. **Start TUI session**
   ```bash
   cargo run --bin ralph -- run --tui -c ralph.claude.yml -p "Write a hello world function in Python"
   ```

2. **Verify initial display**
   - [ ] Header shows `[iter 1/1]` and `[LIVE]`
   - [ ] Hat emoji and name appear (e.g., "🔨 Builder")
   - [ ] Elapsed time ticking
   - [ ] Footer shows activity indicator `◉`

3. **Verify content appears**
   - [ ] Text output appears in content area
   - [ ] Tool calls show with `⚙️` icon
   - [ ] Formatting matches non-TUI output

4. **Wait for iteration 2 (or trigger manually)**
   - [ ] Header updates to `[iter 2/2]`
   - [ ] Previous content preserved

5. **Test navigation**
   - [ ] Press `←` - header shows `[iter 1/2]` and `[REVIEW]`
   - [ ] Content shows iteration 1 output
   - [ ] Footer shows "▶ New: iter 2" alert
   - [ ] Press `→` - returns to `[iter 2/2]`, `[LIVE]`
   - [ ] Alert clears

6. **Test scroll**
   - [ ] Press `j` - content scrolls down
   - [ ] Press `k` - content scrolls up
   - [ ] Press `G` - jumps to bottom
   - [ ] Press `g` - jumps to top

7. **Test search**
   - [ ] Press `/` - footer shows search input
   - [ ] Type "def" - matches highlighted
   - [ ] Press `n` - jumps to next match
   - [ ] Press `N` - jumps to previous match
   - [ ] Press `Esc` - clears search

8. **Test help**
   - [ ] Press `?` - help overlay appears
   - [ ] Press `Esc` - help dismisses

9. **Test exit**
   - [ ] Press `q` - TUI exits cleanly
   - [ ] Terminal restored properly

**Expected Results:**
- All checkboxes above pass
- Output content matches what non-TUI mode would show
- Navigation is smooth and intuitive
- No crashes or visual artifacts

---

## Implementation Plan

### Phase 1: Foundation (Steps 1-3)

#### Step 1: Create IterationBuffer

**Files:** `ralph-tui/src/state.rs`
**Tests:** `test_new_buffer_is_empty`, `test_append_line`, `test_line_count`, `test_visible_lines_*`, `test_scroll_*`
**Demo:** `cargo test -p ralph-tui iteration_buffer` passes

```
RED: Write failing tests for IterationBuffer
GREEN: Implement IterationBuffer struct
REFACTOR: Clean up helper methods
```

#### Step 2: Create TuiStreamHandler

**Files:** `ralph-adapters/src/stream_handler.rs`
**Tests:** `test_on_text_*`, `test_on_tool_call_*`, `test_on_error_*`, `test_parity_*`
**Integrates with:** Step 1 (uses IterationBuffer)
**Demo:** `cargo test -p ralph-adapters tui_stream` passes, parity tests pass

```
RED: Write failing tests for StreamHandler impl
GREEN: Implement TuiStreamHandler
REFACTOR: Extract shared formatting utilities
```

#### Step 3: Refactor TuiState

**Files:** `ralph-tui/src/state.rs`
**Tests:** `test_start_new_iteration`, `test_current_iteration`, `test_navigate_*`, `test_following_latest_*`
**Integrates with:** Step 1 (manages Vec<IterationBuffer>)
**Demo:** `cargo test -p ralph-tui tui_state` passes

```
RED: Write failing tests for iteration management
GREEN: Refactor TuiState with new fields
REFACTOR: Remove deprecated fields (pending_hat, in_scroll_mode)
```

### Phase 2: Widgets (Steps 4-6)

#### Step 4: Create ContentPane

**Files:** `ralph-tui/src/widgets/content.rs`, `ralph-tui/src/widgets/mod.rs`
**Tests:** `test_renders_lines`, `test_respects_scroll`, `test_empty_buffer`
**Integrates with:** Step 1 (renders IterationBuffer)
**Demo:** Unit tests pass, manual render shows styled content

```
RED: Write failing widget render tests
GREEN: Implement ContentPane widget
REFACTOR: Optimize line slicing
```

#### Step 5: Update Header Widget

**Files:** `ralph-tui/src/widgets/header.rs`
**Tests:** `test_iter_position_format`, `test_mode_live`, `test_mode_review`
**Integrates with:** Step 3 (reads TuiState)
**Demo:** Header shows `[iter N/M]` and mode indicator

```
RED: Add failing tests for new format
GREEN: Update render function
REFACTOR: Adjust progressive disclosure
```

#### Step 6: Update Footer Widget

**Files:** `ralph-tui/src/widgets/footer.rs`
**Tests:** `test_new_iter_alert`, `test_no_alert_when_following`, `test_search_query_shown`
**Integrates with:** Step 3 (reads TuiState)
**Demo:** Footer shows new iteration alert, search query

```
RED: Add failing tests for alerts
GREEN: Update render function
REFACTOR: Layout adjustments
```

### Phase 3: Interactions (Steps 7-10)

#### Step 7: Implement Navigation

**Files:** `ralph-tui/src/state.rs`
**Tests:** Already covered in Step 3
**Integrates with:** Step 3 (navigate methods)
**Demo:** Navigation methods work correctly in isolation

```
GREEN: Methods already implemented in Step 3
REFACTOR: Ensure bounds checking is robust
```

#### Step 8: Implement Scroll

**Files:** `ralph-tui/src/state.rs` (IterationBuffer)
**Tests:** Already covered in Step 1
**Integrates with:** Step 4 (ContentPane uses scroll_offset)
**Demo:** Scroll methods work correctly in isolation

```
GREEN: Methods already implemented in Step 1
REFACTOR: Add viewport height awareness
```

#### Step 9: Implement Search

**Files:** `ralph-tui/src/state.rs`
**Tests:** `test_search_finds_matches`, `test_next_match_cycles`, `test_search_jumps_to_match`
**Integrates with:** Step 4 (ContentPane highlights), Step 6 (Footer shows query)
**Demo:** Search finds matches, navigation works

```
RED: Write failing search tests
GREEN: Implement SearchState and methods
REFACTOR: Optimize search algorithm
```

#### Step 10: Simplify Input Handling

**Files:** `ralph-tui/src/input.rs`
**Tests:** All `test_*` input router tests
**Integrates with:** Steps 7-9 (actions trigger state changes)
**Demo:** All key mappings work, no prefix key needed

```
RED: Write comprehensive input tests
GREEN: Rewrite input handler
REFACTOR: Remove old InputRouter code
```

### Phase 4: Integration (Steps 11-14)

#### Step 11: Wire App Event Loop

**Files:** `ralph-tui/src/app.rs`
**Tests:** `test_app_receives_events`, `test_app_keyboard_actions`
**Integrates with:** All previous steps
**Demo:** TUI runs with mock events, renders correctly

```
RED: Write integration tests
GREEN: Refactor app.rs to use new components
REFACTOR: Remove PTY handling code
```

#### Step 12: Update CLI Integration

**Files:** `ralph-cli/src/main.rs`
**Tests:** Smoke test with `ralph run --tui`
**Integrates with:** Step 2 (TuiStreamHandler), Step 11 (App)
**Demo:** Full TUI works with live Claude session

```
RED: Manual smoke test identifies wiring issues
GREEN: Wire TuiStreamHandler into CLI
REFACTOR: Clean up unused PTY setup code
```

#### Step 13: Remove Deprecated Code

**Files:**
- DELETE: `ralph-tui/src/widgets/terminal.rs`
- MODIFY: `ralph-tui/src/scroll.rs` (extract to IterationBuffer or delete)
- MODIFY: `ralph-tui/Cargo.toml` (remove tui-term)
- MODIFY: `ralph-tui/src/widgets/mod.rs`
**Tests:** `cargo build`, `cargo clippy`, `cargo test`
**Demo:** Clean build, reduced binary size

```
GREEN: Remove files and dependencies
REFACTOR: Final cleanup pass
```

#### Step 14: Final Validation

**Files:** Specs, documentation
**Tests:** Full E2E scenario (manual), all automated tests
**Demo:** Complete TUI demo, recorded for PR

```
Execute E2E scenario checklist
Record asciinema demo
Update documentation
```

---

## Test Coverage Summary

| Category | Tests | Description |
|----------|-------|-------------|
| Unit Tests | ~50 | Component isolation tests |
| Integration Tests | ~6 | Component interaction tests |
| E2E Scenario | 1 | Manual validation checklist |

## Implementation Order Rationale

1. **Foundation first** (Steps 1-3): Data structures must exist before they can be used
2. **Widgets second** (Steps 4-6): Visual components need data structures
3. **Interactions third** (Steps 7-10): User interactions need widgets to display results
4. **Integration last** (Steps 11-14): Final wiring after all components work

This order ensures:
- Each step builds on previous work
- Tests can verify functionality incrementally
- Broken functionality is detected early
- No orphaned code (every step produces demoable functionality)

## Success Criteria

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] E2E scenario checklist complete
- [ ] Output matches non-TUI mode formatting
- [ ] `cargo build` clean (no warnings)
- [ ] `cargo clippy` clean
- [ ] `cargo test` passes including smoke tests
- [ ] Binary size reduced (tui-term removed)
