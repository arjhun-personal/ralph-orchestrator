# Terminal Emulator Crates Comparison

## Recommendation: tui-term

**Primary choice for Ralph's TUI terminal embedding.**

### Why tui-term?

1. **Already solved the problem** - designed specifically for ratatui + terminal emulation
2. **Lowest integration cost** - ratatui widget, not raw parser
3. **Uses vt100 underneath** - benefits from proven ANSI parsing
4. **Actively maintained** - part of ratatui ecosystem
5. **Shortest path to shipping** - working embedded terminal in 1-2 weeks

## Crate Comparison

### vt100 (Currently in Ralph)
- **Purpose:** VT100 terminal parser
- **Strengths:** Very lightweight, 1M+ downloads, mature
- **Screen Buffer:** `Screen` struct with cell-level access
- **Scrollback:** `set_scrollback(rows)` for history navigation
- **Use in Ralph:** Currently used for ANSI stripping in `pty_executor.rs`

### alacritty_terminal
- **Purpose:** Full terminal emulation from Alacritty
- **Strengths:** Production-grade, comprehensive VT102+ support
- **Weaknesses:** Overkill for embedded TUI, complex API
- **Recommendation:** Not needed for Ralph's use case

### tui-term
- **Purpose:** Pseudoterminal widget for ratatui
- **Backend:** Uses vt100 internally
- **Integration:** Direct ratatui widget, minimal wrapper code
- **Status:** Active development, work in progress

## Integration Difficulty

| Crate | Difficulty | Custom Code | Time to Ship |
|-------|-----------|------------|-------------|
| **vt100** | Low | Medium (widget wrapper) | 1-2 weeks |
| **alacritty_terminal** | Medium | High (complex API) | 2-3 weeks |
| **tui-term** | Very Low | Minimal (ready-made widget) | 1 week |

## Implementation Path

1. Add tui-term to ralph-tui dependencies
2. Create TerminalWidget component wrapping tui-term
3. Connect to PtyExecutor output for live bytes
4. Handle scrollback through inherited vt100 support
5. Test with CLI tool output

## Sources

- [vt100 crate docs](https://docs.rs/vt100/latest/vt100/)
- [tui-term GitHub](https://github.com/a-kenji/tui-term)
- [alacritty_terminal docs](https://docs.rs/alacritty_terminal/latest/alacritty_terminal/)
- [ratatui Pseudoterminal Discussion](https://github.com/ratatui/ratatui/discussions/540)
