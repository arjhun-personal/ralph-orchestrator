# Cost-free E2E strategy (repo research)

## Goal
Run `ralph-e2e` scenarios without invoking paid AI backends.

## Constraints from current E2E harness
- `ralph-e2e` always spawns `ralph run` with a `ralph.yml` in an isolated workspace.
- Backend availability/auth checks use `<backend> --version` and `which <backend>`. (See `crates/ralph-e2e/src/auth.rs`.)
- `ralph run` always uses PTY streaming (`PtyExecutor`) and expects output that `EventParser` can parse.
- `ralph-e2e` reads events from `.ralph/current-events` → JSONL produced by `EventLogger` (not from stdout).

## Existing building blocks
- **Custom backend support**: `cli.backend: custom` with `command`, `args`, `prompt_mode`, `prompt_flag` is already supported by `CliBackend::custom`.
- **SessionRecorder/SessionPlayer**: JSONL session recordings can be replayed; `SessionPlayer::replay_terminal` replays terminal writes. (`crates/ralph-core/src/session_recorder.rs`, `session_player.rs`.)
- **ReplayBackend**: replays terminal writes from JSONL, but it is used in smoke tests, not in `ralph-e2e`.
- **Cassettes**: `cassettes/` contains JSONL recordings with `ux.terminal.write` and `bus.publish` records (likely SessionRecorder output).
- **ralph-bench replay**: CLI for replaying session JSONL to stdout (`crates/ralph-bench`).
- **Behavioral verification spec** references a mock/cassette backend concept, including mock responses and replay. (`specs/behavioral-verification.spec.md`, `specs/behaviors.yaml`)

## Options for cost-free E2E
### Option A — Custom backend + mock CLI (minimal code changes)
Use `cli.backend: custom` in E2E configs, pointing to a **local mock CLI** that:
- Accepts prompt via args or stdin (to match `prompt_mode`)
- Emits deterministic stdout that includes `<event ...>` XML and completion markers
- Implements `--version` (for E2E availability checks)

Pros:
- Uses existing config path, minimal changes to orchestrator
- Keeps `ralph-e2e` flow intact

Cons:
- Requires a new mock CLI binary/script and a prompt-to-response mapping
- Must be PTY-friendly (stdout streaming)

### Option B — Add first-class `mock` backend to `ralph-e2e`
Introduce a new `Backend::Mock` with:
- Availability check that always passes (or checks for mock binary)
- Scenario setup writes `cli.backend: custom` and points to mock CLI

Pros:
- Cleaner UX for E2E: `ralph-e2e --backend mock`
- Avoids confusing “custom backend” leakage into test configs

Cons:
- Requires changes in `Backend` enum, auth checks, CLI parsing, and scenario selection

### Option C — In-process replay backend for `ralph run`
Add a new built-in backend (e.g., `backend: replay`) that bypasses PTY and uses `ReplayBackend` or `SessionPlayer` directly.

Pros:
- Avoids spawning a child process
- Can reuse JSONL fixtures directly

Cons:
- Requires orchestrator changes and a new backend path
- Higher integration risk; deviates from “thin orchestrator” philosophy

## Mock CLI behavior needed to satisfy E2E assertions
Most E2E scenarios assert:
- stdout is non-empty
- exit code is 0 or 2
- specific substrings (e.g., "PONG", "Builder role activated")
- specific events emitted (`EventLogger` writes JSONL based on parsed `<event>` tags)

A mock CLI can be **rule-based**:
- Extract any `<event ...>...</event>` blocks from the prompt and echo them
- Output required keywords when present in prompt (e.g., `PONG`, `LOOP_COMPLETE`)
- Include a minimal persona string when hat instructions demand it

This keeps logic deterministic and cost-free while exercising the event loop.

## Notes
- `ralph-e2e` already sets `CLAUDE_MODEL=haiku` to reduce cost, but that still calls real APIs.
- If mock CLI is used, ensure it provides `--version` and exits 0 to satisfy availability checks.
