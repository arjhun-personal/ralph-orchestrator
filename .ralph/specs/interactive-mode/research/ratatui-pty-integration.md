# Ratatui PTY Integration Patterns

## Recommended Architecture

The Rust ecosystem has converged on a layered pattern:

```
┌─────────────────────────────────────┐
│ Ratatui (TUI Rendering)             │
├─────────────────────────────────────┤
│ tui-term (Widget Layer)             │
├─────────────────────────────────────┤
│ vt100 (Terminal Parser)             │
├─────────────────────────────────────┤
│ portable-pty (Cross-platform PTY)   │
└─────────────────────────────────────┘
```

## Primary Crates

| Crate | Purpose | Notes |
|-------|---------|-------|
| **tui-term** | PTY widget for ratatui | Purpose-built, actively maintained |
| **portable-pty** | Cross-platform PTY spawning | Used by WezTerm, already in Ralph |
| **vt100** | Terminal escape sequence parser | Extracts grid state, colors, cursor |
| **tui-scrollview** | Scroll position management | For scrollback navigation |

## Input Routing Pattern

```rust
match event {
    // App control (prefix commands)
    Key::Ctrl('a') => handle_prefix_mode(),
    // Everything else goes to PTY
    _ => pty_writer.write(event_bytes),
}
```

Key insight: Library handles terminal state parsing; consumers handle input routing.

## Scrollback Buffer Management

- vt100 maintains grid state internally
- Stores both on-screen and off-screen history
- Use tui-scrollview alongside tui-term for scroll position

## Reference Projects

1. **tui-term** - Minimal, focused library for ratatui
2. **Ratterm** - Full split-terminal TUI with PTY + code editor
3. **ratatui-testlib** - PTY-based test execution harness

## Sources

- [tui-term GitHub](https://github.com/a-kenji/tui-term)
- [Ratterm GitHub](https://github.com/hastur-dev/ratterm)
- [portable-pty docs](https://docs.rs/portable-pty/latest/portable_pty/)
