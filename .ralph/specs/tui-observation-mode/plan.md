# Implementation Plan - TUI Observation Mode

## Test Plan

### Unit Tests

#### Tests to Remove (validate removal was successful)
| File | Test Name | Reason |
|------|-----------|--------|
| `input.rs` | `pause_command_returns_p` | Pause command removed |
| `input.rs` | `skip_command_returns_n` | Skip command removed |
| `input.rs` | `abort_command_returns_a` | Abort command removed |
| `header.rs` | `header_shows_paused_mode` | LoopMode::Paused removed |

#### Tests to Update (adapt to new behavior)
| File | Test Name | Change |
|------|-----------|--------|
| `header.rs` | `create_full_state()` helper | Remove `loop_mode` assignment |
| `header.rs` | All progressive disclosure tests | Remove `loop_mode` field |

#### Tests That Remain Unchanged
- All scroll mode tests (`scroll_mode_*`)
- All search tests (`search_mode_*`, `scroll_mode_enters_*_search`)
- Quit and Help command tests
- Header tests for iteration, elapsed time, hat, idle, scroll indicator

### Integration Tests

#### CLI Argument Tests (new assertions)
- `--tui` flag enables TUI mode
- `-i` flag is rejected with "Found argument '-i' which wasn't expected"
- `--interactive` flag is rejected similarly

### E2E Test Scenario (Manual Verification)

**Prerequisites**: Built binary with `cargo build`

**Steps**:
1. Start TUI: `cargo run --bin ralph -- run --tui -c test-config.yml -p "test prompt"`
   - **Expected**: TUI launches, shows header with iteration, elapsed time, hat, "▶ auto" mode
2. Press `Ctrl+a p` (former pause command)
   - **Expected**: Nothing happens (no mode change, no "⏸ paused" appears)
3. Press `Ctrl+a n` (former skip command)
   - **Expected**: Nothing happens (no skip sent to PTY)
4. Press `Ctrl+a a` (former abort command)
   - **Expected**: Nothing happens (loop continues normally)
5. Press `Ctrl+a [` to enter scroll mode
   - **Expected**: "[SCROLL]" indicator appears in header
6. Press `/` to enter search, type "error", press Enter
   - **Expected**: Search highlights matches, `n`/`N` navigation works
7. Press `q` to exit scroll mode, then `Ctrl+a q` to quit
   - **Expected**: Clean TUI exit, terminal restored

**Pass Criteria**: All 7 steps produce expected results.

---

## Implementation Plan

### Step 1: Update CLI Arguments
**Files**: `crates/ralph-cli/src/main.rs`

**Changes**:
- Remove `interactive: bool` field from `RunArgs` (lines 280-282)
- Remove `tui: bool` hidden alias from `RunArgs` (lines 310-312)
- Add new `tui: bool` field with `#[arg(long, conflicts_with = "autonomous")]`
- Update any references from `args.interactive || args.tui` to just `args.tui`
- Repeat for `ResumeArgs` (lines 325-351)

**Tests that should pass**: `cargo build` succeeds (CLI compiles)

**Demo**: `ralph run --help` shows `--tui` flag, no `-i` flag

---

### Step 2: Remove Command Variants and Routing
**Files**: `crates/ralph-tui/src/input.rs`

**Changes**:
- Remove `Pause`, `Skip`, `Abort` from `Command` enum (lines 19-21)
- Remove match arms for `'p'`, `'n'`, `'a'` in key routing (lines 82-84)
- Remove tests: `pause_command_returns_p`, `skip_command_returns_n`, `abort_command_returns_a` (lines 234-261)

**Tests that should pass**: All remaining input router tests pass

**Demo**: Pressing `Ctrl+a p` in AwaitingCommand mode returns `Command::Unknown`

---

### Step 3: Remove LoopMode Enum and State Field
**Files**:
- `crates/ralph-tui/src/state.rs`
- `crates/ralph-tui/src/lib.rs`

**Changes in state.rs**:
- Remove `LoopMode` enum definition (lines 7-12)
- Remove `pub loop_mode: LoopMode` field from `TuiState` (line 33)
- Remove `loop_mode: LoopMode::Auto` from `new()` initialization (line 62)
- Remove from `with_hat_map()` initialization if present

**Changes in lib.rs**:
- Remove `LoopMode` from public exports (line 24: `pub use state::{LoopMode, TuiState}` → `pub use state::TuiState`)

**Tests that should pass**: State module compiles (will temporarily break header)

**Demo**: N/A (intermediate step, breaks header temporarily)

---

### Step 4: Remove Command Handlers from App
**Files**: `crates/ralph-tui/src/app.rs`

**Changes**:
- Remove `Command::Pause` handler (lines 248-254) that toggles `loop_mode`
- Remove `Command::Skip` handler (lines 255-257) that sends `ControlCommand::Skip`
- Remove `Command::Abort` handler (lines 258-260) that sends `ControlCommand::Abort`
- Remove pause check in PTY forwarding (lines 232-240): delete `is_paused` check, always forward input

**Tests that should pass**: App module compiles

**Demo**: N/A (intermediate step)

---

### Step 5: Update Header Widget
**Files**: `crates/ralph-tui/src/widgets/header.rs`

**Changes**:
- Remove `LoopMode` import (line 1)
- Simplify mode display (lines 75-88): always show "▶ auto" (full) or "▶" (compressed)
- Remove `state.loop_mode` match expression, replace with hardcoded auto display
- Update `create_full_state()` helper to remove `loop_mode` assignment (line 327)
- Remove `header_shows_paused_mode` test (lines 233-244)

**Tests that should pass**: All remaining header tests pass

**Demo**: Header always shows "▶ auto" regardless of any user input

---

### Step 6: Update Help Widget
**Files**: `crates/ralph-tui/src/widgets/help.rs`

**Changes**:
- Remove pause line: `"  p", "  Pause/resume loop"` (lines 32-35)
- Remove skip line: `"  n", "  Skip to next iteration"` (lines 36-39)
- Remove abort line: `"  a", "  Abort loop"` (lines 40-43)
- Add scroll mode documentation section (per design.md requirement R4.2)

**Tests that should pass**: Help widget compiles

**Demo**: `Ctrl+a ?` shows help without pause/skip/abort, includes scroll/search docs

---

### Step 7: Update Example and Behavior Specs
**Files**:
- `crates/ralph-tui/examples/validate_widgets.rs`
- `specs/behaviors.yaml` (if CLI tests exist)

**Changes in validate_widgets.rs**:
- Remove paused mode test case (lines 80-95 approximately)
- Remove `LoopMode` import
- Update any state initialization that sets `loop_mode`

**Changes in behaviors.yaml**:
- Update any CLI flag tests from `-i`/`--interactive` to `--tui`

**Tests that should pass**: `cargo test -p ralph-tui --example validate_widgets`

**Demo**: Example runs without errors

---

### Step 8: Documentation Updates
**Files**: 21+ files across `docs/`, `README.md`, `CLAUDE.md`, `specs/`, `presets/`

**Changes**:
- Replace `-i` with `--tui` in all command examples
- Replace `--interactive` with `--tui` where used
- Replace "interactive mode" terminology with "TUI mode" or "observation mode"

**Search pattern**: `grep -rn "\-i\b\|--interactive" --include="*.md"`

**Tests that should pass**: No functional tests (documentation only)

**Demo**: `grep -rn "\-i " --include="*.md"` returns no results related to TUI flag

---

## Success Criteria

| ID | Criterion | Verification |
|----|-----------|--------------|
| S1 | `ralph run --tui` launches TUI | Manual test |
| S2 | `ralph run -i` fails with error | Manual test |
| S3 | `Ctrl+a p/n/a` do nothing in TUI | Manual E2E test |
| S4 | Scroll mode works (`Ctrl+a [`) | Manual E2E test |
| S5 | Search works (`/`, `?`, `n/N`) | Manual E2E test |
| S6 | All `cargo test` passes | `cargo test` |
| S7 | No `-i`/`--interactive` in docs | grep verification |

## Estimated TDD Cycles

- **Step 1**: 1 cycle (CLI change, test with `--help`)
- **Step 2**: 1 cycle (remove tests, remove code, verify remaining tests)
- **Step 3**: 1 cycle (remove enum, temporary compile errors expected)
- **Step 4**: 1 cycle (remove handlers, fixes compile errors)
- **Step 5**: 1 cycle (update header, remove test, verify rendering)
- **Step 6**: 1 cycle (update help text)
- **Step 7**: 1 cycle (update example and behavior specs)
- **Step 8**: 1 cycle (mechanical doc updates, grep verification)

**Total**: 8 TDD cycles, ~8 implementation tasks
