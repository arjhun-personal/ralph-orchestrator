# TUI Refactor - Rough Idea

## Original Vision

Simplify the TUI crate to show the same output when a user runs ralph in non-TUI mode. The TUI mode is solely to improve UX and tracking and no longer for interacting with the underlying agent.

## Key Points

- When running ralph in TUI mode with claude, the same pretty output is displayed, but with the additional ratatui context widgets
- When an iteration completes, the agent pane is cleared for the next iteration to provide output
- TUI is for observation and context, not interaction

## Goals

1. Unified output experience between TUI and non-TUI modes
2. TUI adds visual enhancements (widgets, context) without changing the core output
3. Clear separation between iterations in TUI mode
