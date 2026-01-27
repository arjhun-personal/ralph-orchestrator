# Interactive TUI Mode - Implementation Plan

## Implementation Checklist

- [x] Step 1: Add tui-term dependency and basic TerminalWidget
- [x] Step 2: Implement PtyHandle abstraction
- [x] Step 3: Wire TerminalWidget to PTY output
- [x] Step 4: Implement InputRouter with prefix detection
- [x] Step 5: Add basic prefix commands (quit, help)
- [x] Step 6: Implement pause/resume loop control
- [x] Step 7: Implement skip and abort commands
- [x] Step 8: Add scroll mode with navigation
- [x] Step 9: Implement search in scroll mode
- [x] Step 10: Handle iteration boundaries (clear screen)
- [x] Step 11: Update header with new fields (idle, mode)
- [x] Step 12: Add TUI configuration to ralph.yml
- [x] Step 13: Deprecate --tui flag, update -i behavior
- [x] Step 14: Integration testing and polish

---

## Step 1: Add tui-term dependency and basic TerminalWidget

**Objective**: Create the foundational terminal widget that can render PTY output.

**Implementation guidance**:
- Add `tui-term` to `ralph-tui/Cargo.toml`
- Create `crates/ralph-tui/src/widgets/terminal.rs`
- Implement basic `TerminalWidget` struct wrapping tui-term's `PseudoTerminal`
- Create a simple test that renders static text through the widget

**Test requirements**:
- Unit test: Create widget, feed bytes, verify screen state
- Manual test: Run standalone binary that shows widget with sample output

**Integration**:
- New file only; no changes to existing code yet

**Demo**: Launch a test binary that displays "Hello from TerminalWidget" in a ratatui frame.

---

## Step 2: Implement PtyHandle abstraction

**Objective**: Create the communication interface between TUI and PTY executor.

**Implementation guidance**:
- Create `crates/ralph-tui/src/pty_handle.rs`
- Define `PtyHandle` struct with channels for input/output/control
- Define `PtyControl` enum (Resize, Terminate)
- Implement async `recv()` for output and sync `write()` for input
- Add trait `OutputHandler` in `pty_executor.rs` to allow pluggable output destinations
- Add trait `InputSource` in `pty_executor.rs` to allow pluggable input sources

**Test requirements**:
- Unit test: Create mock channels, verify send/receive works
- Unit test: Verify control messages are delivered

**Integration**:
- Modifies `pty_executor.rs` to add traits (backward compatible)
- New files in ralph-tui

**Demo**: Test binary that creates PtyHandle, sends "test" through it, receives it back via loopback.

---

## Step 3: Wire TerminalWidget to PTY output

**Objective**: Connect real PTY output to the terminal widget for live rendering.

**Implementation guidance**:
- Modify `TuiApp` to accept `PtyHandle` in constructor
- Add tokio task to read from `pty_handle.recv()` and feed to `TerminalWidget`
- Implement basic render loop that shows PTY output in the terminal pane
- Keep existing header/footer widgets; add terminal widget in middle section

**Test requirements**:
- Integration test: Spawn `echo "hello"` via PTY, verify widget shows output
- Manual test: Run TUI with simple command, see output stream

**Integration**:
- Modifies `ralph-tui/src/app.rs` to add PTY task
- Modifies `ralph-tui/src/lib.rs` to accept PtyHandle

**Demo**: Launch TUI that runs `ls -la` and displays the output in real-time.

---

## Step 4: Implement InputRouter with prefix detection

**Objective**: Route keystrokes correctly between TUI commands and PTY.

**Implementation guidance**:
- Create `crates/ralph-tui/src/input.rs`
- Implement `InputRouter` with `InputMode` state machine
- Handle prefix key detection (default `Ctrl+a`)
- In normal mode, return `InputAction::ForwardToPty`
- In prefix mode, wait for command key
- Wire into TUI event loop

**Test requirements**:
- Unit test: Normal keystroke → ForwardToPty
- Unit test: Prefix key → enters prefix mode
- Unit test: Prefix + unknown key → exits prefix mode, ignores
- Unit test: Prefix + 'q' → returns Quit action

**Integration**:
- New file; modifies app.rs event handling

**Demo**: Type in TUI, see characters appear in PTY. Press Ctrl+a, nothing happens until next key.

---

## Step 5: Add basic prefix commands (quit, help)

**Objective**: Implement the first usable prefix commands.

**Implementation guidance**:
- Handle `Ctrl+a q` → clean shutdown
- Handle `Ctrl+a ?` → show help overlay
- Create `HelpOverlay` widget showing keybinding list
- Implement overlay rendering (modal on top of terminal)

**Test requirements**:
- Integration test: Ctrl+a q exits cleanly
- Integration test: Ctrl+a ? shows overlay, any key dismisses
- Verify terminal state restored after quit

**Integration**:
- Modifies input.rs and app.rs
- New file: widgets/help_overlay.rs

**Demo**: Press Ctrl+a ?, see help popup. Press Ctrl+a q, TUI exits cleanly.

---

## Step 6: Implement pause/resume loop control

**Objective**: Allow users to pause the orchestration loop.

**Implementation guidance**:
- Add `LoopMode` enum to `TuiState` (Auto, Paused)
- Handle `Ctrl+a p` → toggle mode
- Emit control event to `EventLoop` to pause/resume
- Update header to show mode indicator (▶ auto / ⏸ paused)
- When paused, idle timeout should not advance

**Test requirements**:
- Unit test: p toggles mode in state
- Integration test: Press p, header shows paused
- Integration test: While paused, iteration doesn't advance

**Integration**:
- Modifies state.rs, header widget
- May need new channel from TUI to EventLoop

**Demo**: Run loop, press Ctrl+a p, see "⏸ paused" in header. Press again, see "▶ auto".

---

## Step 7: Implement skip and abort commands

**Objective**: Complete the loop control command set.

**Implementation guidance**:
- Handle `Ctrl+a n` → skip to next iteration
  - Send terminate signal to current PTY
  - EventLoop advances to next iteration
- Handle `Ctrl+a a` → abort loop entirely
  - Terminate PTY
  - Signal EventLoop to stop
  - Exit TUI

**Test requirements**:
- Integration test: Ctrl+a n terminates current agent, starts next
- Integration test: Ctrl+a a exits loop and TUI

**Integration**:
- Modifies input handling, adds control channel to EventLoop

**Demo**: Start long-running iteration, press Ctrl+a n, watch it skip. Press Ctrl+a a to abort.

---

## Step 8: Add scroll mode with navigation

**Objective**: Implement scrollback navigation.

**Implementation guidance**:
- Create `ScrollManager` in `crates/ralph-tui/src/scroll.rs`
- Handle `Ctrl+a [` → enter scroll mode
- In scroll mode, capture navigation keys (j/k, arrows, Page Up/Down, g/G)
- Render scroll position indicator
- Exit on q, Escape, or Enter

**Test requirements**:
- Unit test: ScrollManager bounds checking
- Unit test: Navigation key handling
- Integration test: Enter scroll mode, navigate, exit

**Integration**:
- New file: scroll.rs
- Modifies input.rs for scroll mode handling
- Modifies terminal widget for scroll offset

**Demo**: Generate lots of output, Ctrl+a [, scroll up/down with j/k, press q to exit.

---

## Step 9: Implement search in scroll mode

**Objective**: Add search functionality to scroll mode.

**Implementation guidance**:
- Handle `/` in scroll mode → enter search input
- Capture search pattern until Enter
- Implement search in ScrollManager
- Highlight matches in terminal widget
- Handle n/N for next/prev match
- Handle `?` for backward search

**Test requirements**:
- Unit test: Search finds matches
- Unit test: Next/prev cycles through matches
- Integration test: Search, see highlights, navigate matches

**Integration**:
- Modifies scroll.rs, terminal widget rendering

**Demo**: In scroll mode, type /error, see matches highlighted, press n to jump between.

---

## Step 10: Handle iteration boundaries (clear screen)

**Objective**: Clean terminal transition between iterations.

**Implementation guidance**:
- Listen for iteration change events from EventBus
- On new iteration, call `terminal_widget.clear()`
- Reset scroll position to bottom
- Update header with new iteration number

**Test requirements**:
- Integration test: Iteration ends, screen clears
- Integration test: New iteration starts fresh

**Integration**:
- Modifies event observer handling in TuiApp

**Demo**: Run multi-iteration loop, watch terminal clear between iterations.

---

## Step 11: Update header with new fields (idle, mode)

**Objective**: Complete the header display with all required information.

**Implementation guidance**:
- Add `idle_timeout_remaining` to TuiState
- Calculate from EventBus events or PTY activity timestamps
- Format header as: `[iter 3/10] 04:32 | 🎯 Executing | idle: 25s | ▶ auto`
- Update countdown in real-time (each render tick)

**Test requirements**:
- Unit test: Header formatting
- Integration test: Idle countdown decreases over time

**Integration**:
- Modifies state.rs, header widget

**Demo**: Watch idle countdown tick down, see it reset when you type.

---

## Step 12: Add TUI configuration to ralph.yml

**Objective**: Make prefix key and other settings configurable.

**Implementation guidance**:
- Add `tui:` section to config schema
- Parse `prefix_key` from config
- Pass config to TuiApp
- Support key names like "ctrl-a", "ctrl-b"

**Test requirements**:
- Unit test: Parse various key formats
- Integration test: Custom prefix key works

**Integration**:
- Modifies ralph-core config parsing
- Modifies TuiApp to use config

**Demo**: Set `prefix_key: ctrl-b` in config, verify Ctrl+b is now the prefix.

---

## Step 13: Deprecate --tui flag, update -i behavior

**Objective**: Finalize the CLI flag changes.

**Implementation guidance**:
- Remove `--tui` flag from clap args (or make it hidden/deprecated)
- `-i` now automatically launches TUI mode
- Add deprecation warning if `--tui` used
- Update CLI help text

**Test requirements**:
- CLI test: `-i` launches TUI
- CLI test: `--tui` shows deprecation warning
- CLI test: `-a` runs headless (no TUI)

**Integration**:
- Modifies ralph-cli/src/main.rs
- Updates specs/interactive-mode.spec.md frontmatter

**Demo**: Run `ralph run -i`, see TUI. Run `ralph run --tui`, see warning + TUI.

---

## Step 14: Integration testing and polish

**Objective**: Comprehensive testing and UX polish.

**Implementation guidance**:
- Test with all backends (Claude, Kiro, Codex, Gemini, Amp)
- Test edge cases: resize, long lines, rapid output
- Profile scrolling performance (target: 60fps)
- Fix any visual glitches
- Update documentation

**Test requirements**:
- E2E test suite covering all acceptance criteria
- Performance benchmark for scrolling
- Manual testing checklist completion

**Integration**:
- Final polish across all components

**Demo**: Full demo workflow: start loop, interact, scroll, search, pause, skip, quit.

---

## Dependencies Between Steps

```
Step 1 (TerminalWidget)
    ↓
Step 2 (PtyHandle) ─────────────┐
    ↓                           │
Step 3 (Wire PTY to Widget) ←───┘
    ↓
Step 4 (InputRouter)
    ↓
Step 5 (Quit/Help) ──┬── Step 6 (Pause) ──┬── Step 7 (Skip/Abort)
                     │                     │
                     └─────────────────────┴──→ Step 8 (Scroll Mode)
                                                      ↓
                                               Step 9 (Search)
                                                      ↓
Step 10 (Iteration Clear) ←─────────────────────────────
    ↓
Step 11 (Header Update)
    ↓
Step 12 (Config)
    ↓
Step 13 (CLI Flags)
    ↓
Step 14 (Polish)
```

## Estimated Timeline

| Phase | Steps | Estimate |
|-------|-------|----------|
| Foundation | 1-3 | 2-3 days |
| Input & Commands | 4-7 | 2-3 days |
| Scroll & Search | 8-9 | 1-2 days |
| Integration | 10-12 | 1-2 days |
| Finalization | 13-14 | 1-2 days |
| **Total** | | **7-12 days** |
