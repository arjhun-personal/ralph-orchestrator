# TUI Refactor - Requirements Clarification

This document captures the Q&A process to refine requirements for the TUI refactor.

---

## Q1: Output Pipeline Approach

**Question:** How should the TUI receive and display the "pretty output"?

**Options:**
- **A) Stream capture**: TUI implements `StreamHandler` trait, receives same events as `PrettyStreamHandler`, renders to a scrollable text buffer
- **B) Redirect stdout**: Capture `PrettyStreamHandler`'s stdout output and display it in the TUI pane
- **C) Shared renderer**: Create a common rendering layer that both TUI and non-TUI use

**Answer:** Option A - Stream capture. TUI implements `StreamHandler` trait, receives same `ClaudeStreamEvent`s as `PrettyStreamHandler`, and renders to a scrollable text buffer using native ratatui styles.

**Rationale:**
- Non-TUI behavior remains completely untouched (zero regression risk)
- Maximum future extensibility - TUI has full control over rendering
- Clean incremental path to shared rendering logic later if needed
- Both handlers consume same event stream, render appropriately for their context

---

## Q2: Iteration Clearing Behavior

**Question:** You mentioned "when an iteration completes, the agent pane is cleared for the next iteration." How should this clearing work?

**Original Options Considered:**
- A) Hard clear - erase buffer completely
- B) Visual separator - divider with scrollable history
- C) Collapse previous - show last N lines, expandable
- D) Configurable

**Answer:** **Option E - Iteration pagination with navigation arrows**

Each iteration gets its own buffer (like browser tabs). Navigation between iterations via arrow keys or keybindings.

**Behavior:**
- Each iteration has its own isolated buffer
- Current/live iteration shown by default
- Arrow keys (or `h`/`l`, `[`/`]`) navigate between iterations
- Previous iterations preserved but hidden until navigated to
- New iteration = new "page", auto-focuses to it
- Header shows position: `[iter 3/5]`

**Benefits:**
- Clean view: Only see one iteration at a time
- Full history: Can review any previous iteration
- Bounded display: No scrolling through mixed output
- Intuitive mental model (like tmux windows)

---

## Q3: Interaction Model

**Question:** You mentioned TUI is "solely to improve UX and tracking, no longer for interacting with the underlying agent." What keyboard interactions should remain?

**Options:**
- **A) Navigation only**: Arrow keys for iterations, scroll within iteration, `q` to quit
- **B) Navigation + search**: Above plus `/` to search within current iteration
- **C) Minimal**: Just `q` to quit (or Ctrl+C), no other interaction
**Answer:** **Option B - Navigation + search**

**Interactions retained:**
- `←`/`→` (or `h`/`l`, `[`/`]`) - Navigate between iterations
- `j`/`k` or scroll - Scroll within current iteration buffer
- `/` - Search within current iteration
- `q` or Ctrl+C - Quit

**Interactions removed:**
- All input forwarding to the agent (no typing to Claude)
- Prefix key system (Ctrl+A) - no longer needed
- Complex mode state machine

---

## Q4: Auto-Focus on New Iteration

**Question:** If you're viewing a previous iteration (e.g., iteration 2) and iteration 4 starts, should the TUI automatically jump to the new iteration?

**Options:**
- **A) Always auto-focus**: Jump to new iteration immediately
- **B) Stay put with indicator**: Stay on current view, show visual indicator (e.g., blinking "New: iter 4" in footer)
- **C) Smart**: Auto-focus if viewing the "latest" iteration, stay put if deliberately reviewing history

**Answer:** **Option B - Stay put with indicator**

When viewing a previous iteration and a new one starts:
- Stay on current iteration (don't interrupt review)
- Show visual indicator in footer (e.g., "▶ iter 4 started" or blinking indicator)
- User can navigate to new iteration with `→` when ready

---

## Q5: Iteration Buffer Limits

**Question:** Should there be a limit on how many iteration buffers are kept in memory?

**Options:**
- **A) Unlimited**: Keep all iterations (typical runs are <20 iterations anyway)
- **B) Rolling window**: Keep last N iterations (e.g., 50), drop oldest
- **C) Memory-based**: Drop oldest when memory exceeds threshold

**Answer:** **Option A - Unlimited**

Keep all iteration buffers in memory. Rationale:
- Typical runs are <20 iterations
- Text buffers are lightweight (~50-200KB per iteration)
- Early iterations matter for debugging
- YAGNI - can add limits later if needed
- Simplest implementation (just `Vec<IterationBuffer>`)

---

## Requirements Clarification Complete

**Status:** ✅ Complete (user confirmed LGTM)

**Summary of Decisions:**
1. Output pipeline: Stream capture (TUI implements `StreamHandler`)
2. Iteration display: Pagination with arrow navigation
3. Interactions: Navigation + search (j/k, ←/→, /, q)
4. New iteration: Stay put + indicator
5. Buffer limits: Unlimited

**Removals:**
- VT100 terminal emulator (tui-term)
- Input forwarding to agent
- Prefix key system (Ctrl+A)
- Complex mode state machine

**Retentions:**
- Header/footer widgets
- Event bus observer
- Help overlay (simplified)

