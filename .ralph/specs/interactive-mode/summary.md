# Interactive TUI Mode - Project Summary

## Overview

This document summarizes the Prompt-Driven Development process for the Interactive TUI Mode feature, which transforms Ralph's interactive mode into a full-screen TUI with an embedded terminal pane.

## Artifacts Created

```
specs/interactive-mode/
├── rough-idea.md                           # Original concept
├── idea-honing.md                          # 13 Q&A clarifications
├── summary.md                              # This document
├── research/
│   ├── zellij-architecture.md              # Zellij patterns study
│   ├── ratatui-pty-integration.md          # Ratatui + PTY patterns
│   ├── current-ralph-tui.md                # Current TUI analysis
│   ├── current-pty-executor.md             # Current PTY analysis
│   └── terminal-emulator-crates.md         # Crate recommendations
├── design/
│   └── detailed-design.md                  # Comprehensive design spec
└── implementation/
    └── plan.md                             # 14-step implementation plan
```

## Key Design Decisions

| Decision | Choice |
|----------|--------|
| **Layout** | Minimal header + full terminal pane |
| **Implementation** | Native TUI with ratatui + tui-term |
| **Prefix key** | `Ctrl+a` (rebindable) |
| **Scroll mode** | tmux-style with vim navigation |
| **Iteration handling** | Clear screen on boundary |
| **CLI flags** | `-i` launches TUI, `--tui` deprecated |

## Architecture Highlights

- **Stack**: ratatui → tui-term → vt100 → portable-pty
- **Pattern**: Observer-based state from EventBus
- **Input routing**: Prefix commands vs PTY forwarding
- **Scrollback**: Three-section buffer (Zellij-inspired)

## Implementation Plan Summary

14 incremental steps, each with working demo:

1. **Foundation** (Steps 1-3): TerminalWidget, PtyHandle, wire to PTY
2. **Commands** (Steps 4-7): InputRouter, quit/help, pause, skip/abort
3. **Scrolling** (Steps 8-9): Scroll mode, search
4. **Integration** (Steps 10-13): Iteration boundaries, header, config, CLI flags
5. **Polish** (Step 14): Testing, performance, documentation

**Estimated timeline**: 7-12 days

## Next Steps

1. **Review design document** at `design/detailed-design.md`
2. **Review implementation plan** at `implementation/plan.md`
3. **Begin implementation** starting with Step 1 (TerminalWidget)
4. **Deprecate old spec** by updating `specs/interactive-mode.spec.md` frontmatter

## Dependencies to Add

```toml
# In ralph-tui/Cargo.toml
tui-term = "0.1"  # Or latest version
```

## Related Specs

| Spec | Relationship |
|------|--------------|
| `specs/event-loop.spec.md` | Pause/resume integration |
| `specs/cli-adapters.spec.md` | Backend flag filtering |

*Note: The old `specs/interactive-mode.spec.md` was removed to prevent confusion. This new TUI spec supersedes it.*
