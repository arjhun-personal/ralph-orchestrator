---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Update Example and Behavior Specs

## Description
Update the widget validation example to remove paused mode test case and update behavior specs to use `--tui` flag instead of `-i`.

## Background
The example file `validate_widgets.rs` demonstrates various TUI states including paused mode. This must be removed. Additionally, behavior specs in `specs/behaviors.yaml` may test CLI flags.

## Reference Documentation
**Required:**
- Design: specs/tui-observation-mode/design.md

**Additional References:**
- specs/tui-observation-mode/context.md (broken windows section)
- specs/tui-observation-mode/plan.md (Step 7)

**Note:** You MUST read the design document before beginning implementation.

## Technical Requirements
1. In `validate_widgets.rs`: Remove paused mode test case (approximately lines 80-95)
2. In `validate_widgets.rs`: Remove `LoopMode` import
3. In `validate_widgets.rs`: Update any state initialization that sets `loop_mode`
4. In `specs/behaviors.yaml`: Update CLI flag tests from `-i`/`--interactive` to `--tui`

## Dependencies
- Task 06 (help updated) - full TUI should compile and work

## Implementation Approach
1. Read `crates/ralph-tui/examples/validate_widgets.rs`
2. Remove LoopMode import and paused test case
3. Update state initialization in remaining test cases
4. Read `specs/behaviors.yaml` and update flag references
5. Run `cargo test -p ralph-tui --example validate_widgets` to verify

## Acceptance Criteria

1. **LoopMode import removed from example**
   - Given validate_widgets.rs is modified
   - When searching for "LoopMode"
   - Then no matches are found

2. **Paused test case removed**
   - Given validate_widgets.rs is modified
   - When searching for "paused" or "Paused"
   - Then no test case matches are found

3. **Example compiles and runs**
   - Given all changes are complete
   - When running `cargo run --example validate_widgets -p ralph-tui`
   - Then example executes without errors

4. **Behavior specs updated**
   - Given specs/behaviors.yaml is modified
   - When searching for "-i" or "--interactive"
   - Then no matches are found (all use `--tui`)

5. **All tests pass**
   - Given all changes are complete
   - When running `cargo test`
   - Then all tests pass

## Metadata
- **Complexity**: Low
- **Labels**: examples, specs, cleanup
- **Required Skills**: Rust, YAML
