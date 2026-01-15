---
status: completed
created: 2026-01-14
started: 2026-01-14
completed: 2026-01-14
---
# Code Task: Fix TUI Visual Issues

## Overview

Fix visual rendering issues in the Ralph TUI identified by the `/tui-validate` skill.

## Context

The TUI validation skill identified the following issues in `ISSUES.md`:
1. Header displays double space after emoji due to emoji width handling
2. Footer uses hardcoded whitespace (30 spaces) for alignment
3. Unused status widget (dead code)

## Requirements

### Issue 1: Fix emoji spacing in header

**File:** `crates/ralph-tui/src/state.rs`

Remove the space between emoji and hat name in all hat display strings:
- `"📋 Planner"` → `"📋Planner"`
- `"🔨 Builder"` → `"🔨Builder"`

This fixes the visual double-space issue since emojis render as double-width characters.

### Issue 2: Remove hardcoded whitespace in footer

**File:** `crates/ralph-tui/src/widgets/footer.rs`

Replace the hardcoded 30-space string with a flexible spacer that adapts to terminal width.

The footer currently uses:
```rust
Span::raw("                              "),  // 30 hardcoded spaces
```

Change the `render` function to accept the render area and use `Constraint::Fill` for responsive layout.

### Issue 3: Remove unused status widget

**Files:**
- `crates/ralph-tui/src/widgets/status.rs` - Delete this file
- `crates/ralph-tui/src/widgets/mod.rs` - Remove `pub mod status;` line

The status widget is dead code that duplicates functionality already in the header.

## Acceptance Criteria

- [ ] Running `cargo run -p ralph-tui --example validate_widgets` shows single space after emoji (e.g., `🔨Builder`)
- [ ] Footer activity indicator aligns properly at different terminal widths
- [ ] `status.rs` is removed and no compilation errors occur
- [ ] All existing tests pass (`cargo test -p ralph-tui`)
- [ ] No clippy warnings (`cargo clippy -p ralph-tui`)

## Complexity

Medium - involves modifying multiple files but changes are straightforward

## Test Plan

1. Run validation example: `cargo run -p ralph-tui --example validate_widgets`
2. Verify header shows `🔨Builder` (no double space)
3. Verify footer layout looks correct
4. Run tests: `cargo test -p ralph-tui`
5. Run clippy: `cargo clippy -p ralph-tui`
