# Requirements Clarification

This document captures the Q&A process for refining the Interactive TUI Mode feature.

---

## Q1: Pane Layout Vision

**Question:** When you envision the TUI with CLI output in its own pane, what layout are you thinking? For example:

- **Two-pane vertical split**: CLI output on top, orchestrator status/controls on bottom
- **Two-pane horizontal split**: CLI output on left, status on right
- **Three-pane layout**: CLI output (main), status sidebar, input area
- **Something else?**

Also, should the CLI output pane be the dominant/largest pane, or should panes be roughly equal?

**Answer:** KISS approach—minimal chrome:

- **Header bar** (single line or small strip) showing:
  - Current iteration number
  - Elapsed time
  - Current "hat" (planning, executing, etc.)
- **Main area**: Full terminal session hosting the CLI agent (Claude, Codex, Gemini, etc.)

No sidebars, no complex pane layouts. The agent terminal is the star; the header provides just enough orchestration context.

---

## Q2: Terminal Session Interactivity

**Question:** For the main terminal pane hosting the CLI agent, should users be able to:

1. **Fully interact** with the agent (type input, respond to prompts, scroll history) just like they would in a normal terminal?
2. **Read-only with intervention points** (watch output, but Ralph controls when to pause for user input)?
3. **Something in between?**

Essentially: is this a "Ralph drives, user watches with context" or "user drives within Ralph's orchestration frame"?

**Answer:** Fully interactive. Users can:

- Type input and respond to agent prompts
- Scroll through history
- Use zellij/tmux-style commands to run commands, exit, etc.

The terminal pane is a real, fully interactive terminal session—not a read-only viewer. Ralph provides the orchestration frame (header with iteration/time/hat), but the user has full control of the agent interaction.

---

## Q3: Implementation Approach—Native TUI vs. Terminal Multiplexer

**Question:** You mentioned "using zellij/tmux to run commands/exit." This could mean two different approaches:

1. **Native TUI**: Ralph renders its own TUI using ratatui, embedding a PTY in the main pane. Zellij/tmux-*like* keybindings for exiting, but Ralph owns the UI.

2. **Actual zellij/tmux integration**: Ralph spawns inside (or spawns) a zellij/tmux session, using their pane management. Ralph just manages the header pane; zellij/tmux handles the terminal pane.

Which approach are you envisioning?

**Answer:** Native TUI. Ralph builds its own TUI using ratatui with an embedded PTY. Zellij and tmux serve as prior art to study for:

- Keybinding conventions
- Interaction patterns
- Terminal embedding techniques
- UX for pane/session management

Ralph owns the entire UI stack—no external multiplexer dependency.

---

## Q4: Key Commands and Exit Behavior

**Question:** For the native TUI, what key commands do you expect? Some possibilities inspired by zellij/tmux:

- **Exit/detach**: `Ctrl+q` to quit Ralph entirely? Or a prefix key like `Ctrl+b` then `d` to detach?
- **Pause iteration**: A way to pause Ralph's loop while staying in the TUI?
- **Scroll mode**: A key to enter scrollback mode (like tmux's `Ctrl+b [`)?
- **Other commands**: Anything else you'd want accessible via keybindings?

What's your preference for the core command set?

**Answer:**

- **Prefix key approach**: Use a prefix key (like tmux's `Ctrl+b`) followed by command keys
- **Ralph loop controls**: Basic controls for the orchestration loop (pause, resume, skip, abort, etc.)
- **Scroll mode**: Yes, with smooth, performant scrolling—must be as good as zellij's performance
- **Performance is critical**: Scrolling and rendering must be buttery smooth, no lag

---

## Q5: Prefix Key and Core Commands

**Question:** Let's nail down the specifics:

1. **Prefix key**: What prefix do you prefer?
   - `Ctrl+r` (for Ralph—easy to remember, not heavily used)
   - `Ctrl+b` (tmux default—familiar to tmux users)
   - `Ctrl+a` (screen default—also common)
   - Something else?

2. **Loop controls after prefix**: Which of these do you want?
   - `p` - Pause/resume the loop
   - `n` - Skip to next iteration
   - `a` - Abort loop entirely
   - `s` - Enter scroll mode
   - `q` - Quit TUI
   - Others?

**Answer:**

- **Prefix key**: `Ctrl+a` (screen default) as the default
- **Rebindable**: Users should be able to rebind the prefix key (and presumably other keys) to their preference via configuration
- **Loop controls**: (Confirming the set in next question)

---

## Q6: Confirming Loop Controls

**Question:** To confirm the loop control commands after `Ctrl+a`, which of these do you want in the initial version?

- `Ctrl+a p` - Pause/resume the loop
- `Ctrl+a n` - Skip to next iteration (force advance)
- `Ctrl+a a` - Abort loop entirely
- `Ctrl+a [` - Enter scroll mode (tmux-style)
- `Ctrl+a q` - Quit TUI and Ralph
- `Ctrl+a ?` - Show help/keybindings

All of the above? A subset? Any additions?

**Answer:** All of the above for the initial version:

- `Ctrl+a p` - Pause/resume the loop
- `Ctrl+a n` - Skip to next iteration (force advance)
- `Ctrl+a a` - Abort loop entirely
- `Ctrl+a [` - Enter scroll mode (tmux-style)
- `Ctrl+a q` - Quit TUI and Ralph
- `Ctrl+a ?` - Show help/keybindings

---

## Q7: Iteration Boundaries

**Question:** When one iteration ends and the next begins, what should happen in the terminal pane?

1. **Clear and fresh**: Terminal clears, new iteration starts with a clean slate
2. **Continuous scroll**: Output continues flowing; a visual separator (line, banner) marks iteration boundaries
3. **User choice**: Configurable behavior

Also, should there be a brief pause or indicator between iterations so the user knows a transition happened?

**Answer:**

- **Clear and fresh**: Terminal clears at iteration boundary, new iteration starts with clean slate
- **No explicit pause needed**: The refresh/clear itself makes the transition apparent
- Scrollback history from previous iterations is discarded (or optionally preserved in logs, but not in the TUI scrollback)

---

## Q8: Relationship with Existing Flags

**Question:** Currently Ralph has:
- `-i, --interactive` flag: Enables PTY with user input (current interactive mode)
- `--tui` flag: Enables ratatui-based monitoring UI

How should the new unified TUI relate to these?

1. **Replace both**: The new TUI becomes the default for `-i` (interactive mode always uses TUI)
2. **New flag**: Add a new flag like `--tui-interactive` or similar, keeping old modes available
3. **`--tui` evolves**: When `-i` and `--tui` are both specified, use the new unified mode
4. **Other approach?**

**Answer:** Option 1 - Replace both. The new TUI becomes the default behavior for `-i` (interactive mode).

- `-i` / `--interactive` now launches the unified TUI with embedded terminal
- The old `--tui` monitoring-only mode is deprecated/removed
- Simpler mental model: interactive = TUI with full terminal, autonomous = headless

---

## Q9: Configuration Location

**Question:** You mentioned keybindings should be rebindable. Where should TUI configuration live?

1. **In existing ralph.toml**: Add a `[tui]` section for keybindings, colors, etc.
2. **Separate file**: e.g., `~/.config/ralph/tui.toml` or `ralph-tui.toml`
3. **Both**: Defaults in ralph.toml, user overrides in a dedicated file

Also, should there be any theming/color configuration, or stick with sensible defaults for now?

**Answer:** In existing `ralph.yml`. Add a `[tui]` or `tui:` section for keybindings and any other TUI config.

- Keep configuration consolidated in one place
- Theming: Sensible defaults for now (not explicitly requested, can add later)

---

## Q10: Header Format Details

**Question:** For the header bar showing iteration/elapsed time/current hat, what format do you envision?

Example layouts:
```
┌─ Ralph ─────────────────────────────────────────────────────────────────┐
│ Iteration: 3/10  │  Elapsed: 00:04:32  │  Hat: 🎯 Executing             │
└─────────────────────────────────────────────────────────────────────────┘
```

Or more minimal:
```
[iter 3/10] 04:32 | 🎯 Executing
```

Or something else? Any specific info you want in the header beyond iteration, time, and hat?

**Answer:** Minimal format with a line separator:

```
[iter 3/10] 04:32 | 🎯 Executing | idle: 25s | ▶ auto
────────────────────────────────────────────────────
```

Header contents:
- Iteration: current/total (or current if unbounded)
- Elapsed time
- Current hat (Planning, Executing, etc.)
- Idle timeout remaining (countdown)
- Mode indicator: ▶ auto (running) or ⏸ paused

Line break/separator between header and terminal pane for visual clarity.

---

## Q11: Error and Edge Cases

**Question:** What should happen in these scenarios?

1. **Agent crashes mid-iteration**: Show error in terminal, auto-advance to next iteration? Pause for user input? Abort?

2. **Idle timeout expires**: Currently terminates the agent. In the TUI, should it show a message and wait, or auto-advance to next iteration?

3. **User hits Ctrl+C in the terminal** (not the prefix command): Forward to agent as interrupt, or intercept for Ralph?

**Answer:**

1. **Agent crashes**: Auto-advance to next iteration. The idle-timeout mechanism handles this naturally—if the agent stops producing output, timeout triggers and advances.

2. **Idle timeout expires**: Auto-advance to next iteration. No manual intervention needed.

3. **Ctrl+C in terminal**: Forward to agent as interrupt, same behavior as a terminal multiplexer. Ralph prefix commands (`Ctrl+a` + key) are the escape hatch for Ralph-level control.

Philosophy: Keep things flowing automatically. Ralph handles iteration advancement; user intervenes only when they want to (via prefix commands).

---

## Q12: Existing Spec Deprecation

**Question:** The existing `specs/interactive-mode.spec.md` covers the current PTY-based interactive mode. How should we handle this?

**Answer:** Deprecate it. The new TUI spec supersedes the existing interactive mode spec.

Key relationships:
- **Existing spec** (`specs/interactive-mode.spec.md`): Covers `-i` flag with raw PTY, signal handling, idle timeout, agent flag filtering
- **New spec**: TUI wraps all of this—the PTY becomes embedded in the TUI, but the underlying mechanics (agent flags, signal handling, idle timeout) remain similar

Action items:
1. ~~Mark `specs/interactive-mode.spec.md` as `status: deprecated`~~ → **Removed** to prevent confusion
2. The new TUI spec incorporates the still-relevant parts (agent flag filtering, idle timeout semantics)

---

## Q13: Scroll Mode Behavior

**Question:** For scroll mode (`Ctrl+a [`), what's the expected behavior?

1. **Exit scroll mode**: Escape? `q`? Any key that's not a navigation key?
2. **Navigation**: Arrow keys? vim-style (j/k/g/G)? Page Up/Down?
3. **Search**: Support `/` to search within scrollback? (Can defer to later)

**Answer:** Follow tmux conventions:

**Exit scroll mode:**
- `q` or `Escape` to exit
- Also exits when pressing `Enter`

**Navigation:**
- Arrow keys (up/down for lines, left/right for horizontal scroll)
- vim-style: `j`/`k` for line, `Ctrl+u`/`Ctrl+d` for half-page, `g`/`G` for top/bottom
- `Page Up`/`Page Down` for full page
- `Home`/`End` for beginning/end of history

**Search:**
- `/` for forward search
- `?` for backward search
- `n`/`N` for next/previous match

This gives vim users and tmux users familiar muscle memory.

---

