# TUI Observation Mode - Requirements

## Summary

Transform the TUI from "interactive mode" to "observation mode" by renaming the flag, removing execution controls, and keeping observation aids.

## Consolidated Requirements

### R1: Flag Rename
- **R1.1**: `--tui` becomes the sole primary flag to enable TUI mode
- **R1.2**: Remove `-i` and `--interactive` entirely (no aliases, no deprecation warnings)
- **R1.3**: Update all documentation, examples, and tests to use `--tui`
- **R1.4**: Applies to both `run` and `resume` subcommands

### R2: Remove Execution Controls
- **R2.1**: Remove Pause (`Ctrl+a p`) - affects execution flow
- **R2.2**: Remove Skip (`Ctrl+a n`) - sends Skip command to PTY
- **R2.3**: Remove Abort (`Ctrl+a a`) - sends Abort command to PTY
- **R2.4**: Remove `LoopMode::Paused` state (no longer needed)
- **R2.5**: Remove pause check in PTY forwarding logic

### R3: Keep Observation Aids
- **R3.1**: Keep Scroll mode (`Ctrl+a [`) - only affects view
- **R3.2**: Keep Mouse scroll - only affects view
- **R3.3**: Keep vim-style navigation (`j/k`, `gg/G`, `Ctrl+u/d`) in scroll mode
- **R3.4**: Keep Search (`/` forward, `?` backward, `n/N` navigation)
- **R3.5**: Keep Help (`Ctrl+a ?`) - displays keyboard shortcuts
- **R3.6**: Keep Quit (`Ctrl+a q`) - clean TUI exit

### R4: Update Help Screen
- **R4.1**: Remove documentation for Pause, Skip, Abort commands
- **R4.2**: Add documentation for scroll mode navigation
- **R4.3**: Add documentation for search functionality

## Design Rationale

### No Backward Compatibility
Per project policy (CLAUDE.md:184): "Backwards compatibility doesn't matter — it adds clutter for no reason"

### Scope Decision
- **Execution controls** (Pause/Skip/Abort) removed: These send commands to the PTY and affect Ralph's execution
- **Observation aids** (Scroll/Search) kept: These only modify local view state with zero impact on Ralph

### Semantic Alignment
- **Current**: TUI = interactive mode (user controls execution)
- **Target**: TUI = observation mode (user watches without interfering)

## Success Criteria

1. `ralph run --tui -c config.yml -p "prompt"` launches visual TUI
2. `-i` and `--interactive` flags are unrecognized (CLI error)
3. No way for users to pause, skip, or abort via keyboard
4. Scroll mode and search work exactly as before
5. All docs/tests pass with `--tui`
6. Help screen shows only observation commands
