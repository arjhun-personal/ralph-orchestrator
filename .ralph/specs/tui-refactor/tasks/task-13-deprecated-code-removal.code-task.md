---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Remove Deprecated Code

## Description
Delete deprecated files, remove unused dependencies, and clean up code that is no longer needed after the refactor. This includes the VT100 terminal widget and tui-term dependency.

## Background
The refactor replaces VT100 terminal emulation with direct line rendering. This means:
- `terminal.rs` widget is no longer needed
- `tui-term` dependency can be removed
- `scroll.rs` may be simplified or merged into IterationBuffer
- Various PTY-related code throughout the codebase

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: Cleanup)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- specs/tui-refactor/research/broken-windows.md (known issues to clean up)

**Note:** You MUST read the broken windows document before beginning.

## Technical Requirements
1. DELETE `ralph-tui/src/widgets/terminal.rs`
2. MODIFY `ralph-tui/src/widgets/mod.rs` — remove terminal widget export
3. MODIFY `ralph-tui/Cargo.toml` — remove tui-term dependency
4. REVIEW `ralph-tui/src/scroll.rs`:
   - If functionality merged into IterationBuffer, DELETE
   - If still needed, simplify
5. Remove any unused imports across modified files
6. Fix any Clippy warnings in touched files

## Dependencies
- Task 11: App event loop wiring (confirms terminal widget unused)
- Task 12: CLI integration update (confirms PTY code unused)

## Implementation Approach
1. **GREEN**: Delete files and remove dependencies
2. **VERIFY**: `cargo build` compiles without errors
3. **VERIFY**: `cargo clippy` has no warnings
4. **REFACTOR**: Final cleanup pass on any remaining issues

## Acceptance Criteria

1. **Terminal Widget Deleted**
   - Given refactored codebase
   - When checking for `terminal.rs`
   - Then file does not exist

2. **tui-term Removed**
   - Given `ralph-tui/Cargo.toml`
   - When examining dependencies
   - Then `tui-term` is not listed

3. **Module Exports Updated**
   - Given `ralph-tui/src/widgets/mod.rs`
   - When examining exports
   - Then no terminal widget is exported

4. **Clean Build**
   - Given all deletions complete
   - When running `cargo build`
   - Then no compilation errors

5. **Clean Clippy**
   - Given all deletions complete
   - When running `cargo clippy`
   - Then no warnings

6. **All Tests Pass**
   - Given the cleanup is complete
   - When running `cargo test`
   - Then all tests pass

7. **Broken Windows Resolved**
   - Given issues from broken-windows.md
   - When examining touched files
   - Then issues are resolved (where applicable to removed code)

## Metadata
- **Complexity**: Low
- **Labels**: cleanup, dependencies, tui
- **Required Skills**: Rust, Cargo dependency management, code removal
