---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Fix TUI Connected Detection Logic

## Description
Fix the `tui_connected` detection logic in the PTY executor that incorrectly determines TUI mode by checking `self.output_rx.is_none()`. Since `handle()` is never called in the current TUI implementation, `output_rx` is always `Some`, causing `tui_connected` to always be `false`. This results in raw ANSI-formatted PTY output being written directly to stdout, corrupting ratatui's alternate screen.

## Background
The PTY executor has a `tui_connected` flag used to determine whether to write raw PTY output to stdout. The original design expected `handle()` to be called when TUI mode is active, which would take ownership of `output_rx` and set it to `None`. However, after the real-time streaming refactor (commit 1a819563), the data flow changed to use shared buffers directly via `TuiStreamHandler`, and `handle()` is no longer called.

**Current broken flow:**
1. TUI mode is enabled with `--tui` flag
2. `execute_pty` is called with `tui_lines: Some(...)`
3. PTY executor checks `self.output_rx.is_none()` → returns `false` (because `handle()` wasn't called)
4. `tui_connected = false` even though TUI IS connected
5. In `run_observe` (non-StreamJson fallback): `if !tui_connected` → `true` → raw ANSI output written to stdout
6. ratatui's alternate screen is corrupted

**Affected code locations:**
- `crates/ralph-adapters/src/pty_executor.rs:326` - `run_observe`
- `crates/ralph-adapters/src/pty_executor.rs:576` - `run_observe_streaming`
- `crates/ralph-adapters/src/pty_executor.rs:842` - `run_interactive`

## Technical Requirements
1. Add an explicit `tui_mode: bool` field to `PtyExecutor` struct
2. Create a method to set the TUI mode flag (e.g., `set_tui_mode(&mut self, enabled: bool)`)
3. Replace all `self.output_rx.is_none()` checks with the explicit `tui_mode` field
4. Update `execute_pty` in `main.rs` to set the TUI mode flag when `tui_lines.is_some()`
5. Ensure the fix works for both StreamJson (Claude) and non-StreamJson (Kiro, Gemini) backends
6. Maintain backward compatibility - non-TUI mode should continue to work as before

## Dependencies
- `crates/ralph-adapters/src/pty_executor.rs` - Primary file to modify
- `crates/ralph-cli/src/main.rs` - `execute_pty` function needs to set TUI mode
- Existing smoke tests in `crates/ralph-core/tests/` for regression testing

## Implementation Approach
1. **Add TUI mode field to PtyExecutor:**
   ```rust
   pub struct PtyExecutor {
       // ... existing fields
       tui_mode: bool,  // New field
   }
   ```

2. **Initialize in constructor:**
   ```rust
   pub fn new(backend: CliBackend, config: PtyConfig) -> Self {
       // ... existing code
       Self {
           // ... existing fields
           tui_mode: false,  // Default to non-TUI mode
       }
   }
   ```

3. **Add setter method:**
   ```rust
   pub fn set_tui_mode(&mut self, enabled: bool) {
       self.tui_mode = enabled;
   }
   ```

4. **Replace detection logic in all three methods:**
   ```rust
   // Before (broken):
   let tui_connected = self.output_rx.is_none();

   // After (fixed):
   let tui_connected = self.tui_mode;
   ```

5. **Update execute_pty in main.rs:**
   ```rust
   if let Some(exec) = executor {
       if tui_lines.is_some() {
           exec.set_tui_mode(true);
       }
       // ... rest of execution
   }
   ```

6. **Run smoke tests to verify no regressions**

## Acceptance Criteria

1. **TUI Mode Detection Works Correctly**
   - Given the TUI is enabled with `--tui` flag
   - When `execute_pty` is called with `tui_lines: Some(...)`
   - Then `tui_connected` evaluates to `true` in all PTY executor methods

2. **No Raw Output in TUI Mode**
   - Given the TUI is running with a non-StreamJson backend (e.g., Kiro)
   - When the PTY executor falls back to `run_observe`
   - Then raw ANSI output is NOT written to stdout

3. **Non-TUI Mode Still Works**
   - Given the application is run without `--tui` flag
   - When PTY execution occurs
   - Then output is written to stdout as before (no regression)

4. **StreamJson Path Unaffected**
   - Given the Claude backend with StreamJson format
   - When running in TUI mode
   - Then JSON parsing and TuiStreamHandler flow works correctly

5. **Smoke Tests Pass**
   - Given the existing smoke test suite
   - When running `cargo test -p ralph-core smoke_runner`
   - Then all tests pass without regression

6. **Unit Tests Added**
   - Given the new `tui_mode` field and `set_tui_mode` method
   - When the test suite runs
   - Then there are unit tests verifying the TUI mode detection logic

## Metadata
- **Complexity**: Low
- **Labels**: Bug Fix, TUI, PTY, ANSI, Terminal
- **Required Skills**: Rust, Terminal handling, ratatui understanding
