# Current PTY Executor Architecture

## PTY Creation

Uses `portable-pty` for cross-platform pseudo-terminal support:
- Creates `PtyPair` with master (host side) and slave (child process side)
- Configurable terminal dimensions (default: 80x24)
- Sets `TERM=xterm-256color` for rich color support

## Data Flow

```
stdin thread → PTY master writer
         ↓
     PTY slave (agent process)
         ↓
PTY master reader → output reader thread → mpsc channel
```

## Execution Modes

**Observe Mode (Synchronous):**
- No user input forwarding
- Single blocking loop reading PTY output
- Used for non-interactive agent execution

**Interactive Mode (Asynchronous):**
- User input forwarded (with special key encoding)
- Two spawned threads: output reader, input reader
- Main tokio task multiplexes via `tokio::select!`

## Signal Handling

**Ctrl+C Double-Press Detection:**
- First press: Forward to agent, start 1-second window
- Second press within window: Terminate agent

**Ctrl+\ (SIGQUIT):**
- Immediate SIGKILL to agent

**Idle Timeout:**
- Resets on PTY output AND user input
- Triggers SIGTERM → 5s grace → SIGKILL

## Raw Terminal Mode

PTY executor does NOT manage raw mode. Handled externally in `cli_backend.rs`:
- `enable_raw_mode()` before PTY executor
- `disable_raw_mode()` via scopeguard on exit

## Changes Needed for TUI Embedding

1. **Extract Output Handler (Trait)**
   - Replace `io::stdout().write_all()` with pluggable handler
   - TUI can inject its own buffer handler

2. **Extract Input Source (Trait)**
   - Replace stdin thread with configurable input source
   - TUI provides events from its event loop

3. **Make Raw Mode Optional**
   - Add config flag: `manage_raw_mode: bool`
   - TUI handles raw mode externally

## Implementation Estimate

- Add `OutputHandler` trait: 1-2 hours
- Extract input source: 1-2 hours
- Add raw mode config flag: 30 min
- Create TUI integration layer: 2-3 hours
- Full integration testing: 2-3 hours

**Total: 7-11 hours for full TUI embedding**
