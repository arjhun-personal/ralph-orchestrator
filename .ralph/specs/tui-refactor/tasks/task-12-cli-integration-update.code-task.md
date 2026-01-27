---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Update CLI Integration

## Description
Wire TuiStreamHandler into the CLI main.rs to replace PTY-based TUI initialization. The CLI should pass TuiStreamHandler to the session runner when `--tui` flag is used.

## Background
Currently, `ralph-cli/src/main.rs` sets up a PTY for TUI mode and connects it to the Claude session. The refactored approach uses TuiStreamHandler as a StreamHandler implementation that writes to TuiState instead of using PTY/VT100.

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: Integration Points)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- `ralph-cli/src/main.rs:1594-1622` — Current TUI setup

**Note:** You MUST read the design document and CLI main.rs before beginning.

## Technical Requirements
1. Modify `ralph-cli/src/main.rs`
2. Replace PTY setup with TuiStreamHandler instantiation
3. Share TuiState between App and TuiStreamHandler via `Arc<Mutex<TuiState>>`
4. Start App event loop in parallel with session runner
5. Clean up unused PTY setup code

## Dependencies
- Task 2: TuiStreamHandler (created and tested)
- Task 11: App event loop wiring (app is ready)

## Implementation Approach
1. **RED**: Manual smoke test identifies wiring issues
2. **GREEN**: Wire TuiStreamHandler into CLI
3. **REFACTOR**: Clean up unused PTY setup code

## Acceptance Criteria

1. **TuiStreamHandler Used in TUI Mode**
   - Given `--tui` flag
   - When session starts
   - Then TuiStreamHandler is the active handler

2. **State Shared Between Components**
   - Given TUI mode active
   - When content arrives via TuiStreamHandler
   - Then App can render the new content

3. **TUI Renders Full Session**
   - Given `ralph run --tui -c config.yml -p "hello"`
   - When session runs to completion
   - Then content is visible in TUI throughout

4. **Non-TUI Mode Unchanged**
   - Given no `--tui` flag
   - When session runs
   - Then PrettyStreamHandler is used (no regression)

5. **Smoke Test Passes**
   - Given the implementation is complete
   - When running `cargo run --bin ralph -- run --tui -c ralph.claude.yml -p "test"`
   - Then TUI displays content and responds to input

## Metadata
- **Complexity**: Medium
- **Labels**: integration, cli, tui
- **Required Skills**: Rust, CLI argument handling, Arc/Mutex patterns
