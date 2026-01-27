# Detailed Design: Cost‑Free E2E Mock Adapter

## Overview

This design adds a **cost‑free, deterministic E2E mode** to the Ralph E2E harness by introducing a **mock CLI adapter** implemented as a new `ralph-e2e mock-cli` subcommand. In mock mode, `ralph-e2e` generates a `ralph.yml` that uses `cli.backend: custom` and points to this mock CLI, which **replays recorded JSONL cassettes** (SessionRecorder format) and optionally executes a **whitelisted set of local commands** to satisfy task/memory side‑effects. The orchestration loop remains intact and still uses PTY streaming, preserving realistic integration behavior without invoking paid APIs.

Key characteristics:
- **Opt‑in mock mode** via a flag (e.g., `ralph-e2e --mock`).
- **Per‑backend matrix preserved** (Claude/Kiro/OpenCode) for reporting parity.
- **Cassette‑driven output** (source of truth) with deterministic naming and fail‑fast behavior.
- **Accelerated replay** by default (no real-time delays).
- **Whitelisted command execution** for tasks/memories side effects.

## Detailed Requirements

### Functional Requirements
1. **Mock mode flag**: `ralph-e2e` must support an opt‑in mock mode flag (e.g., `--mock`).
2. **Mock CLI adapter**: Implement a new `ralph-e2e mock-cli` subcommand used as a custom backend command.
3. **Cassette source of truth**: Mock CLI must replay output from JSONL cassettes (SessionRecorder format).
4. **Cassette naming**: Use `cassettes/e2e/<scenario-id>-<backend>.jsonl` with fallback to `cassettes/e2e/<scenario-id>.jsonl`.
5. **Fail‑fast on missing cassette**: Missing cassette must cause the scenario to fail with a clear error.
6. **Accelerated replay**: Mock replay should be accelerated by default (no real timing delays).
7. **Whitelist command execution**: Mock CLI may execute a restricted set of local commands (e.g., `ralph task add`, `ralph tools memory add`) to satisfy side‑effect assertions.
8. **Per‑backend matrix**: In mock mode, run each scenario for each backend in the existing matrix.
9. **Bypass backend availability/auth**: In mock mode, `ralph-e2e` should bypass real backend installation/auth checks.
10. **No injected iteration markers**: Mock CLI should not add iteration separators; replay output as‑is.
11. **Error scenario coverage**: Mock mode should run error scenarios using dedicated cassettes and controls.
12. **Interface**: Mock CLI must support `--cassette <path>`, `--speed <n>`, and a whitelist mechanism (flag or env).

### Non‑Functional Requirements
- Deterministic output for CI stability.
- No external network usage in mock mode.
- Minimal changes to core orchestrator logic.
- Clear error messages for missing cassettes or disallowed commands.

## Architecture Overview

```mermaid
flowchart LR
  E2E[ralph-e2e --mock] --> Runner[Scenario Runner]
  Runner --> Workspace[Per-scenario workspace]
  Runner -->|writes| Config[ralph.yml (custom backend)]
  Runner --> Exec[RalphExecutor]
  Exec --> Ralph[ralph run]
  Ralph -->|PTY| MockCLI[ralph-e2e mock-cli]
  MockCLI -->|replay JSONL| Stdout[stdout stream]
  MockCLI -->|optional| LocalCmds[whitelisted local commands]
  Ralph --> EventLoop[EventLoop + EventParser]
  EventLoop --> Events[.ralph/events-*.jsonl]
  Exec --> Assertions[Scenario assertions]
```

## Components and Interfaces

### 1) `ralph-e2e` CLI additions
- **New flag**: `--mock` (boolean) on `ralph-e2e` command.
- **New subcommand**: `mock-cli` (used as external command for `cli.backend: custom`).

#### Proposed CLI shape
```
ralph-e2e --mock [--cassette-dir cassettes/e2e] [--mock-speed 10.0]

ralph-e2e mock-cli --cassette <path> --speed <n> [--allow "ralph task add,ralph tools memory add"]
```

(Defaults defined in code: cassette dir fixed to `cassettes/e2e/`.)

### 2) Scenario setup changes (mock mode)
In `TestScenario::setup`, when mock mode is enabled:
- Write `ralph.yml` with `cli.backend: custom` and `cli.command` set to the **absolute path** of the `ralph-e2e` binary.
- Set `cli.args` to invoke `mock-cli` with `--cassette <resolved path>` and `--speed <n>`.
- Preserve other `event_loop`, `hats`, and `memories` config as-is.

### 3) Cassette resolution
Given a scenario id (e.g., `events`) and backend (`claude`), compute:
1. `cassettes/e2e/<scenario-id>-<backend>.jsonl`
2. If missing: `cassettes/e2e/<scenario-id>.jsonl`
3. If missing: **fail fast** with explicit error.

### 4) Mock CLI behavior
The `mock-cli` subcommand performs:
- If invoked with `--version`, print a static version string and exit 0 (to satisfy availability checks when needed).
- Load cassette JSONL via `SessionPlayer` (reuse `ralph-core` types).
- Replay only `ux.terminal.write` records to stdout using **text mode** or terminal mode. (Text mode preferred for deterministic output; ANSI preserved if cassette contains it.)
- Replay speed: `--speed` defaults to accelerated (e.g., 10x or “instant”).
- While replaying, **scan output stream** for tool-use instructions, and if allowed by whitelist, execute local commands.

#### Command whitelist execution
- Input: `--allow "ralph task add,ralph tools memory add"` or env var `RALPH_MOCK_ALLOW`.
- Only exact prefix matches allowed; no shell evaluation.
- Execute via `Command::new("ralph")` with direct args.
- Run in current working directory (scenario workspace).
- Capture stdout/stderr and echo to mock stdout.
- Enforce short timeout per command (e.g., 5s).

### 5) Error scenario support
Dedicated cassettes for:
- `timeout-handling` (simulate long output or explicit delay if needed by mock).
- `max-iterations` (cassette outputs but no completion promise, allowing max-iteration exit).
- `auth-failure` (cassette emits stderr text with auth keywords; optionally exit code != 0).
- `backend-unavailable` (cassette simulates missing backend by exiting non-zero and emitting appropriate stderr).

### 6) Runner behavior in mock mode
- Skip AuthChecker availability/auth checks entirely when `--mock` is enabled.
- Still run per‑backend matrix and record results by backend name.

## Data Models

### Cassette record format (existing)
```json
{"ts":1000,"event":"ux.terminal.write","data":{"bytes":"...base64...","stdout":true,"offset_ms":0}}
```
- `SessionPlayer` and `ReplayBackend` already parse this.
- `mock-cli` will reuse `SessionPlayer` to avoid new parsers.

### Mock CLI config
```text
--cassette <path>   # required
--speed <n>         # optional (default accelerated)
--allow <csv>       # optional whitelist
```

## Error Handling
- Missing cassette: print clear error to stderr and exit non‑zero.
- Cassette parse error: emit error and exit non‑zero.
- Disallowed command attempt: log to stderr, continue replay (or fail if configured).
- Whitelisted command failure: surface stdout/stderr and exit non‑zero for error scenarios if needed.

## Testing Strategy

### Unit tests
- Cassette resolution logic (scenario+backend → correct path).
- Mock CLI parsing of flags and env.
- Whitelist enforcement logic.
- Mock CLI `--version` output.

### Integration tests
- Run `ralph-e2e --mock` on a small subset of scenarios with test cassettes.
- Ensure `events` scenario emits correct events in `.ralph/events-*.jsonl`.
- Ensure `task-add` and `memory-add` scenarios create the expected files via whitelist execution.

### Regression
- Existing real‑backend E2E should remain unchanged when `--mock` is not used.

## Appendices

### A. Technology Choices
- **Reuse `SessionPlayer`** for cassette playback to minimize new code.
- **Mock CLI within `ralph-e2e`** for packaging simplicity.
- **Custom backend** instead of new adapter to keep orchestrator unchanged.

### B. Research Findings (summary)
- `ralph run` always uses PTY streaming; mock CLI must tolerate PTY.
- `ralph-e2e` reads events from `.ralph/current-events` JSONL (EventLogger output).
- Tasks/memory scenarios require side effects; whitelist command execution is needed.

### C. Alternatives Considered
- Built‑in mock backend in `ralph-adapters`: more invasive and bypasses PTY integration.
- In‑process replay backend: diverges from real execution path.

