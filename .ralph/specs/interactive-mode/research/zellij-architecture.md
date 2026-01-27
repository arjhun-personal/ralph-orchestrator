# Zellij Architecture Research

## PTY Embedding and Session Management

**Zellij's Approach:**
- **Client-Server Architecture**: Client and server run as separate processes communicating via Unix sockets
- **Multi-threaded Server Design**: Dedicated threads for screen rendering, terminal I/O, plugins, and background jobs communicate via MPSC channels
- **Async I/O**: Uses Tokio runtime for non-blocking PTY I/O handling

## Scrollback and Viewport Architecture

**Three-section buffer design:**
- `lines_above`: Scrollback buffer (lines scrolled out of view)
- `viewport`: Currently visible lines
- `lines_below`: Lines below viewport when scrolled up

**Key optimization:**
- Lazy line wrapping: Only wraps lines in the active viewport during terminal resize
- Avoids expensive rewrapping of 10K+ line scrollback buffers
- Lines move between sections on scroll events (O(1) per line operation)

## Keybinding and Modal Input System

**Modal Architecture:**
- Modes organize keybindings into context-specific groups (normal, pane, locked, etc.)
- Two keybinding presets:
  - **Default**: Modal access from normal mode (Ctrl+p for pane mode)
  - **Locked Mode**: Requires prefix key unlock (Ctrl+g as prefix)

## Key Patterns for Ralph

1. **Scrollback Strategy**: Implement three-section buffer model (lines_above/viewport/lines_below)
2. **Rendering Efficiency**: Segregate high-frequency terminal I/O from low-frequency control events
3. **Keybinding Design**: Prefix-key over modal system for simplicity
4. **Memory Trade-offs**: Accept higher per-pane memory cost for feature richness

## Sources

- [Zellij Official Website](https://zellij.dev/)
- [Zellij GitHub Repository](https://github.com/zellij-org/zellij)
- [Building Zellij's Web Client](https://poor.dev/blog/building-zellij-web-terminal/)
