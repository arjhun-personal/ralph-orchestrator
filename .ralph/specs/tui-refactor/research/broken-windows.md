# Broken Windows - TUI Refactor

Low-risk code smells found in files that will be modified during implementation.

## Summary

| File | Issues | Risk |
|------|--------|------|
| `state.rs` | 2 | Low |
| `input.rs` | 1 | Low |
| `app.rs` | 2 | Low |
| `stream_handler.rs` | 1 | Low |
| **Total** | **6** | Low |

---

## crates/ralph-tui/src/state.rs

### [state.rs:104] Duplicated fallback logic

**Type:** duplication
**Risk:** Low
**Fix:** Extract to a helper function or simplify with early returns

**Code:**
```rust
// Lines 101-134: Hardcoded topic matching duplicates custom hat map logic
match topic {
    "task.start" => {
        // Save hat_map before resetting
        let saved_hat_map = std::mem::take(&mut self.hat_map);
        *self = Self::new();
        self.hat_map = saved_hat_map;
        // ...
    }
```

**Note:** During the refactor, this entire state model is being redesigned. The broken window will be resolved naturally.

---

## crates/ralph-tui/src/input.rs

### [input.rs:47-50] Hardcoded default prefix

**Type:** magic-values
**Risk:** Low
**Fix:** Define as constants

**Code:**
```rust
Self {
    mode: InputMode::Normal,
    prefix_key: KeyCode::Char('a'),       // Magic value
    prefix_modifiers: KeyModifiers::CONTROL,  // Magic value
}
```

**Suggested fix:**
```rust
const DEFAULT_PREFIX_KEY: KeyCode = KeyCode::Char('a');
const DEFAULT_PREFIX_MODIFIERS: KeyModifiers = KeyModifiers::CONTROL;
```

**Note:** The prefix key system is being removed in this refactor. This broken window will be resolved automatically.

---

## crates/ralph-tui/src/app.rs

### [app.rs:110] Clippy lint suppression

**Type:** complexity
**Risk:** Low
**Fix:** The function is being refactored anyway - this lint suppression will be removed

**Code:**
```rust
#[allow(clippy::too_many_lines)]
pub async fn run(mut self) -> Result<()> {
```

**Note:** The simplified event loop in the new design should be significantly shorter.

### [app.rs:50-51] Dead code annotation

**Type:** dead-code
**Risk:** Low
**Fix:** Remove annotation after verifying usage

**Code:**
```rust
#[allow(dead_code)] // Public API - may be used by external callers
pub fn new(state: Arc<Mutex<TuiState>>, pty_handle: PtyHandle) -> Self {
```

**Note:** The `App::new()` function is currently unused internally - only `App::with_prefix()` is called. The annotation is defensive but adds noise. The refactored App should reassess which constructors are truly needed.

---

## crates/ralph-adapters/src/stream_handler.rs

### [stream_handler.rs:258-269] Duplicated truncate function

**Type:** duplication
**Risk:** Low
**Fix:** The same `truncate()` helper exists in `claude_stream.rs:115-128` with slightly different implementations

**Code in stream_handler.rs:**
```rust
fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let byte_idx = s.char_indices().nth(max_len).map(|(idx, _)| idx).unwrap_or(s.len());
        format!("{}...", &s[..byte_idx])
    }
}
```

**Code in claude_stream.rs:**
```rust
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {  // Note: uses len() not chars().count()
        s.to_string()
    } else {
        let boundary = s.char_indices()
            .take_while(|(i, _)| *i < max_len)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!("{}...", &s[..boundary])
    }
}
```

**Note:** The `claude_stream.rs` version has a bug - it uses byte length for the initial check but character-based truncation. The `stream_handler.rs` version is more consistent. Consider extracting to a shared utility during refactor.

---

## Files Without Issues

These files were reviewed and found to be clean:

- `crates/ralph-tui/src/widgets/header.rs` - Well-structured with progressive disclosure
- `crates/ralph-tui/src/widgets/footer.rs` - Clean implementation
- `crates/ralph-tui/src/scroll.rs` - Clean implementation (will be partially reused)
- `crates/ralph-tui/src/lib.rs` - Clean public interface

---

## Recommendations

1. **Fix during refactor:** All identified issues are in files being significantly modified. Fix them as part of the refactor rather than as separate commits.

2. **Skip:** The `truncate()` duplication is outside the core TUI files. Consider consolidating into `ralph-core` as a follow-up task, but don't let it block this refactor.

3. **Builder MAY fix:** The identified issues are low-risk and can be addressed during the REFACTOR phase of TDD if time permits.
