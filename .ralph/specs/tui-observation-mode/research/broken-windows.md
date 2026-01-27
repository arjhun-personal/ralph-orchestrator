# Broken Windows - TUI Observation Mode

Low-risk code smells identified in touched files that MAY be fixed during refactor phase.

## validate_widgets.rs

### [crates/ralph-tui/examples/validate_widgets.rs:81] Paused mode test case
**Type**: dead-code (after LoopMode removal)
**Risk**: Low
**Fix**: Remove the entire "paused mode" rendering test case (lines 80-95)
**Code**:
```rust
// Render header with paused mode (1-line borderless design)
state.loop_mode = ralph_tui::LoopMode::Paused;
let backend = TestBackend::new(80, 1);
...
```

**Note**: This becomes dead code after `LoopMode::Paused` is removed. The example should still test Auto mode, but the Paused test case should be deleted.

## header.rs

### [crates/ralph-tui/src/widgets/header.rs:24] Dead code marker
**Type**: dead-code
**Risk**: Low
**Fix**: Remove `#[allow(dead_code)]` once WIDTH_HIDE_HELP is used or remove the constant
**Code**:
```rust
#[allow(dead_code)] // Kept for documentation of breakpoint tiers
const WIDTH_HIDE_HELP: u16 = 65; // Below this: help hint hidden
```

**Note**: This constant exists for documentation but isn't used in conditional logic. The help hint is only shown at WIDTH_FULL (80+). Consider either using it or documenting the breakpoint tiers elsewhere.

### [crates/ralph-tui/src/widgets/header.rs:221-244] Paused mode tests
**Type**: dead-code (after LoopMode removal)
**Risk**: Low
**Fix**: Remove `header_shows_paused_mode` test entirely
**Code**:
```rust
#[test]
fn header_shows_paused_mode() {
    let mut state = TuiState::new();
    state.loop_mode = LoopMode::Paused;
    ...
}
```

## state.rs

### [crates/ralph-tui/src/state.rs:73-91] Duplicated state initialization
**Type**: duplication
**Risk**: Low
**Fix**: Consider using a builder pattern or `Default::default()` to reduce repetition between `new()` and `with_hat_map()`
**Code**:
```rust
pub fn with_hat_map(hat_map: HashMap<String, (HatId, String)>) -> Self {
    Self {
        pending_hat: None,        // Same as new()
        iteration: 0,             // Same as new()
        prev_iteration: 0,        // Same as new()
        ...
    }
}
```

**Note**: This is a minor improvement opportunity. After removing `loop_mode`, both functions will have slightly less repetition. Could use `Self { hat_map, ..Default::default() }` if `Default` is derived.

## Summary

| File | Count | Types |
|------|-------|-------|
| validate_widgets.rs | 1 | dead-code |
| header.rs | 2 | dead-code |
| state.rs | 1 | duplication |

**Total**: 4 low-risk fixes identified.

**Recommendation**: The dead-code fixes (3) are directly caused by this change and MUST be addressed. The duplication fix is optional but would simplify future maintenance.
