# Implementation Plan

- [x] Step 1: Add mock mode flag + cassette resolution helpers in ralph-e2e
- [x] Step 2: Add `mock-cli` subcommand in ralph-e2e
- [x] Step 3: Wire mock mode into scenario setup + runner (custom backend config)
- [x] Step 4: Implement whitelist command execution + error handling
- [x] Step 5: Add tests and example cassettes; validate mock E2E subset

## Implementation Status

**Completed**: All 5 steps implemented. Mock mode is fully functional for single-iteration scenarios.

**Cassettes created**:
- `connect.jsonl` - Connectivity test
- `events.jsonl` - Event parsing test
- `completion.jsonl` - LOOP_COMPLETE detection
- `single-iter.jsonl` - Single iteration (note: scratchpad assertion not supported)
- `multi-iter.jsonl` - Multi-iteration (note: multi-iteration not supported due to architecture)

**Known limitations**:
1. Multi-iteration scenarios: Mock-cli replays entire cassette in one invocation, so Ralph only sees one iteration
2. Scratchpad assertions: Require file writes which aren't in the default whitelist
3. Task/Memory scenarios: Work when cassettes include `ralph task`/`ralph tools memory` commands in bus.publish events

## Step 1: Add mock mode flag + cassette resolution helpers in ralph-e2e
**Objective:** Introduce a `--mock` flag and deterministic cassette selection logic.

**Implementation Guidance:**
- Extend `ralph-e2e` CLI args to include `--mock` and `--mock-speed` (default accelerated).
- Add a helper to resolve cassette path by scenario id + backend:
  - `cassettes/e2e/<scenario-id>-<backend>.jsonl`
  - fallback `cassettes/e2e/<scenario-id>.jsonl`
  - fail if missing
- Ensure cassette resolution is used only in mock mode.

**Tests:**
- Unit tests for cassette resolution behavior (specific vs fallback vs missing).

**Integration:**
- No runtime behavior change unless `--mock` is enabled.

**Demo:**
- `ralph-e2e --mock --list` shows scenarios will use cassette paths (via debug logs or dry-run output).

---

## Step 2: Add `mock-cli` subcommand in ralph-e2e
**Objective:** Create a replaying mock CLI that can be used as a custom backend command.

**Implementation Guidance:**
- Add `mock-cli` subcommand with flags: `--cassette <path>`, `--speed <n>`, `--allow <csv>`.
- Implement `--version` handler that prints a static version and exits 0.
- Use `SessionPlayer` to load cassette and replay `ux.terminal.write` events to stdout.
- Default to accelerated replay (speed multiplier or “instant”).

**Tests:**
- Unit test: `mock-cli --version` returns exit 0.
- Unit test: replay outputs expected text for a minimal cassette.

**Integration:**
- Standalone invocation: `ralph-e2e mock-cli --cassette <file>` writes replay to stdout.

**Demo:**
- `ralph-e2e mock-cli --cassette cassettes/e2e/events.jsonl --speed 10` prints deterministic output.

---

## Step 3: Wire mock mode into scenario setup + runner
**Objective:** Use `mock-cli` in E2E runs when `--mock` is enabled.

**Implementation Guidance:**
- In scenario `setup`, if mock mode enabled:
  - Write `cli.backend: custom` in `ralph.yml`.
  - Set `command` to the `ralph-e2e` binary path.
  - Set `args` to `mock-cli --cassette <path> --speed <n>`.
- Ensure per-backend matrix remains intact.
- Bypass backend availability/auth checks when `--mock` is enabled.

**Tests:**
- Unit test: mock mode writes correct ralph.yml (custom backend config).
- Unit test: availability/auth checks are skipped in mock mode.

**Integration:**
- Running `ralph-e2e --mock` uses mock CLI in workspaces.

**Demo:**
- `ralph-e2e --mock --backend claude` runs without real CLI installed.

---

## Step 4: Implement whitelist command execution + error handling
**Objective:** Support task/memory side‑effects and error scenarios in mock mode.

**Implementation Guidance:**
- Parse replay output (or prompt) to detect whitelisted command instructions.
- Execute only whitelisted command prefixes (e.g., `ralph task add`, `ralph tools memory add`).
- Capture stdout/stderr and echo into mock output stream.
- Enforce timeout per command; fail if command errors when required by cassette.
- Ensure disallowed command attempts are surfaced clearly.

**Tests:**
- Unit test: whitelist allows approved commands.
- Unit test: disallowed command rejected.
- Unit test: command output is forwarded to stdout.

**Integration:**
- Task/memory scenarios in mock mode create `.agent/tasks.jsonl` / `.agent/memories.md`.

**Demo:**
- `ralph-e2e --mock --backend claude task-add` passes task assertions without API calls.

---

## Step 5: Add tests and example cassettes; validate mock E2E subset
**Objective:** Ensure mock mode is deterministic and CI‑safe.

**Implementation Guidance:**
- Add minimal cassettes under `cassettes/e2e/` for a subset (e.g., connect, events, task-add).
- Ensure cassettes contain `ux.terminal.write` outputs matching expected assertions.
- Add tests that run a small subset of scenarios with `--mock`.

**Tests:**
- Unit tests for cassette load/replay.
- Integration test for mock mode on 1–2 scenarios.

**Integration:**
- Confirm `cargo test` passes and `ralph-e2e --mock` runs without real CLI.

**Demo:**
- `ralph-e2e --mock --backend claude connect` runs fully offline.

