# Design: `ralph bot daemon`

## Problem

When no Ralph loop is running, Telegram messages go nowhere. Users must SSH in or open a terminal to start `ralph run`. The daemon bridges this gap — a persistent process that listens on Telegram and starts loops on demand.

## Behavior

### State Machine

```
          ┌─────────────┐
          │    Idle      │◄──────────────────┐
          │  (polling)   │                   │
          └──────┬───────┘                   │
                 │ message received          │ loop finishes
                 ▼                           │
          ┌──────────────┐    ┌──────────────┴──┐
          │ Check lock   │───▶│  Loop Running    │
          │ locked?  ────│──▶ │  (interaction)   │
          │ unlocked? ───│──▶ │  (start loop)    │
          └──────────────┘    └─────────────────┘
```

**Idle (no loop running):**
- Polls Telegram via `getUpdates` with 30s long-poll timeout
- On message: checks `LoopLock::is_locked()`
- If unlocked: starts a new loop with message as prompt
- If locked (e.g., loop started externally): routes as guidance/response

**Loop running:**
- Normal interaction model — messages become `human.guidance` or `human.response` events
- Replies to pending questions route as `human.response`
- Freeform messages route as `human.guidance`
- Same behavior as existing Telegram integration during `ralph run`

**Loop finishes:**
- Daemon sends completion message via Telegram
- Returns to idle, waits for next message

### Loop Startup

- Daemon sends ack: "Starting loop: *{prompt}*"
- Spawns loop in a **tokio task** so Telegram polling continues concurrently
- Loop runs in **main workspace** (not a worktree)
- Config loaded from `ralph.yml` (no passthrough flags)
- Lock acquired normally via `LoopLock::try_acquire()`

### Special Commands

| Command | Behavior |
|---------|----------|
| `/stop` | Cancels running loop (via `CancellationToken`), daemon stays alive |
| `/status` | Reports: loop running/idle, current hat, iteration count |

### Startup & Shutdown

- `ralph bot daemon` starts the daemon
- Sends greeting: "Ralph daemon online"
- On Ctrl+C / SIGTERM: sends farewell, cleans up, exits

## Implementation

### Step 1: Extract loop-startup logic

In `crates/ralph-cli/src/main.rs`, extract from `run_command()`:

```rust
pub async fn start_loop(
    prompt: String,
    workspace_root: PathBuf,
    config_path: Option<PathBuf>,
) -> Result<()>
```

Handles: load config, apply prompt, acquire lock, run event loop, release lock. Existing `run_command()` becomes a thin wrapper.

### Step 2: Add `daemon` subcommand

In `crates/ralph-cli/src/bot.rs`:

- Add `Daemon` variant to `BotSubcommand` enum
- `run_daemon()` function:
  - Resolve bot token (env → keychain → config)
  - Load chat_id from `.ralph/telegram-state.json`
  - Send greeting
  - Enter polling loop:
    - Long-poll `getUpdates`
    - On `/stop`: cancel running loop task
    - On `/status`: reply with state
    - On regular message: check lock, start loop or route interaction
  - Spawn loop as tokio task for concurrent polling
  - On shutdown: send farewell, clean up

### Step 3: Loop completion notifications

When spawned loop task finishes:
- Success: "Loop complete."
- Error: "Loop failed: {error}"
- Daemon returns to idle state

## Non-Goals

- Worktree isolation for daemon-spawned loops
- CLI flags passthrough to spawned loops
- Queuing multiple loops
- Confirmation before starting a loop

## Files Changed

| File | Change |
|------|--------|
| `crates/ralph-cli/src/main.rs` | Extract `start_loop()` from `run_command()` |
| `crates/ralph-cli/src/bot.rs` | Add `Daemon` subcommand, `run_daemon()` |
| `crates/ralph-telegram/src/` | Minor refactoring to support daemon lifecycle |
