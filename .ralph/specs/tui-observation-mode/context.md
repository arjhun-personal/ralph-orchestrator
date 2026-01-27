# Implementation Context - TUI Observation Mode

## Summary

This change is **purely subtractive** - removing execution controls (Pause, Skip, Abort) from the TUI while keeping observation aids (Scroll, Search). The primary challenge is ensuring all 42+ documentation references are updated atomically.

## Key Files to Modify

### Core Changes (5 files)
| File | Change | Lines |
|------|--------|-------|
| `crates/ralph-cli/src/main.rs` | Remove `-i/--interactive`, make `--tui` primary | 280-312, 325-351 |
| `crates/ralph-tui/src/input.rs` | Remove `Command::{Pause,Skip,Abort}`, key routing | 16-24, 79-87 |
| `crates/ralph-tui/src/state.rs` | Remove `LoopMode` enum and `loop_mode` field | 7-12, 33, 62 |
| `crates/ralph-tui/src/app.rs` | Remove command handlers and pause check | 232-260 |
| `crates/ralph-tui/src/widgets/help.rs` | Update help text | 18-53 |

### Secondary Changes (5 files)
| File | Change | Reason |
|------|--------|--------|
| `crates/ralph-tui/src/lib.rs` | Remove `LoopMode` from public exports | Line 24 |
| `crates/ralph-tui/src/widgets/header.rs` | Remove `LoopMode::Paused` match arms | 77-88 |
| `crates/ralph-tui/examples/validate_widgets.rs` | Remove paused mode test | 80-95 |
| `specs/behaviors.yaml` | Update CLI flag behavior tests | 31-59 |
| 21+ documentation files | Replace `-i`/`--interactive` with `--tui` | Various |

## Patterns to Follow

### Input Routing
Keep the prefix command pattern (`Ctrl+a` then key). Simply remove three match arms:
```rust
// Remove these three lines from the match:
'p' => Command::Pause,
'n' => Command::Skip,
'a' => Command::Abort,
```

### CLI Arguments
Use clap's standard pattern:
```rust
// Before: hidden deprecated alias
#[arg(long, hide = true)]
tui: bool,

// After: primary flag (remove hide, keep conflicts_with)
#[arg(long, conflicts_with = "autonomous")]
tui: bool,
```

### State Management
Complete removal pattern - remove enum, field, and all usages:
1. Delete `LoopMode` enum definition
2. Remove `loop_mode` field from `TuiState`
3. Remove from `new()` and `with_hat_map()` initialization
4. Remove public export from `lib.rs`

## Integration Points

### PTY Control Channel
The TUI sends commands via `control_tx`:
```rust
// app.rs - These lines are removed, but the channel remains for future use
let _ = self.control_tx.send(ControlCommand::Skip);
let _ = self.control_tx.send(ControlCommand::Abort);
```

The `ralph-adapters` crate still defines these commands - they're just no longer sent from TUI.

### Header Widget
After removing `LoopMode::Paused`, the mode display simplifies:
```rust
// Before: match on Auto/Paused
// After: always show running state (▶ auto / ▶)
```

**Decision**: Keep showing "▶ auto" to indicate the loop is running. This preserves the visual indicator without the pause option.

## Constraints Discovered

### 1. Public API Change
`LoopMode` is publicly exported. Removing it is a breaking change, but per project policy (CLAUDE.md:184), backwards compatibility doesn't matter.

### 2. Example File Updates
`validate_widgets.rs` uses `LoopMode::Paused` for testing. The paused mode test case must be removed (broken window identified).

### 3. Behavior Specs
`specs/behaviors.yaml` has 8 CLI flag tests referencing `-i/--interactive`. These MUST be updated to test `--tui` instead.

### 4. Documentation Volume
42+ occurrences of `-i`/`--interactive` across 21 files. Recommend:
1. Use grep/sed for mechanical replacement
2. Verify each change makes sense in context
3. Update examples to show `--tui` flag

## Test Impact

### Tests to Remove
| File | Test Name |
|------|-----------|
| `input.rs` | `pause_command_returns_p` |
| `input.rs` | `skip_command_returns_n` |
| `input.rs` | `abort_command_returns_a` |
| `header.rs` | `header_shows_paused_mode` |

### Tests to Update
- Header tests that reference `LoopMode::Paused` in `create_full_state()` helper
- Any integration tests checking for pause/skip/abort behavior

### Tests That Remain Unchanged
- Scroll mode tests (preserved feature)
- Search tests (preserved feature)
- Help display tests (update content, keep test structure)
- Quit command test (preserved feature)

## Broken Windows Summary

4 low-risk fixes identified:
1. `validate_widgets.rs:81` - Remove paused mode test case (REQUIRED)
2. `header.rs:24` - Dead code constant (OPTIONAL)
3. `header.rs:221-244` - Remove paused mode test (REQUIRED)
4. `state.rs:73-91` - Duplicated initialization (OPTIONAL)

The REQUIRED fixes are direct consequences of this change. OPTIONAL fixes improve code quality but aren't blocking.

## Implementation Order Recommendation

1. **CLI args** (`main.rs`) - Change flag, breaking change early
2. **Input routing** (`input.rs`) - Remove command variants and routing
3. **State** (`state.rs`, `lib.rs`) - Remove LoopMode entirely
4. **App handlers** (`app.rs`) - Remove command handlers and pause check
5. **Widgets** (`header.rs`, `help.rs`) - Update display
6. **Example** (`validate_widgets.rs`) - Remove paused test
7. **Behavior specs** (`behaviors.yaml`) - Update CLI tests
8. **Documentation** (21+ files) - Mechanical replacement

This order ensures type errors are caught early (removing `LoopMode` before removing its usages would cause compile errors, so we work from usage sites toward definition).
