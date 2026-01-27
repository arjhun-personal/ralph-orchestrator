# Event loop + headless execution flow (repo study)

## Key files reviewed
- crates/ralph-cli/src/loop_runner.rs
- crates/ralph-core/src/event_loop/mod.rs
- crates/ralph-core/src/event_parser.rs
- crates/ralph-adapters/src/cli_backend.rs
- crates/ralph-adapters/src/pty_executor.rs

## Summary of the flow
- `ralph run` calls `run_loop_impl` (ralph-cli). It resolves the prompt, sets up process group leadership, initializes `EventLoop`, configures `EventLogger`, and optionally a `SessionRecorder` for JSONL replay.
- Execution mode is computed from `cli.default_mode` and terminal detection. PTY execution is always enabled for streaming output.
- TUI is observation-only. It uses the same backend execution path but swaps stream handlers to render output in the terminal UI.
- Per iteration:
  - `EventLoop::next_hat()` determines the active hat; `EventLoop::build_prompt()` creates the prompt.
  - `PtyExecutor` runs the backend command (always under PTY), streaming output.
  - For Claude `stream-json`, NDJSON lines are parsed and text blocks are accumulated into `extracted_text` for event parsing. For text backends, ANSI-stripped output is used directly.
  - `EventParser` extracts `<event topic="...">payload</event>` tags from output.
  - `EventLoop::process_output()` updates state and routes events; it also reads JSONL events from `.ralph/events-*.jsonl` written by agents.
- Termination conditions are checked in `EventLoop::check_termination()` and `EventLoop::process_output()`; PTY termination is mapped to `TerminationReason` in `run_loop_impl`.

## Data flow diagram
```mermaid
sequenceDiagram
  participant CLI as ralph-cli run_loop_impl
  participant Loop as ralph-core EventLoop
  participant Backend as ralph-adapters CliBackend
  participant Pty as PtyExecutor
  participant Tool as External CLI (claude/kiro/opencode/custom)

  CLI->>Loop: initialize(prompt)
  loop iteration
    Loop->>CLI: next_hat + build_prompt
    CLI->>Pty: run_observe_streaming(prompt)
    Pty->>Backend: build_command
    Backend-->>Pty: command + args + prompt_mode
    Pty->>Tool: spawn via PTY
    Tool-->>Pty: output stream (text or NDJSON)
    Pty-->>CLI: output + extracted_text
    CLI->>Loop: process_output (EventParser)
    Loop-->>Loop: route events + update state
  end
```

## Headless-mode behaviors that affect mocks
- PTY is always used, even in autonomous mode. Any mock CLI must tolerate PTY execution.
- Claude `stream-json` output is parsed line-by-line. Event tags must appear inside assistant text blocks so they are visible to `EventParser`.
- For text backends, `EventParser` consumes ANSI-stripped output, so event tags can be emitted directly in plain text.
