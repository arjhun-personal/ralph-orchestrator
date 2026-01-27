# E2E scenario needs (high-level)

This summarizes what the mock backend must produce to satisfy current `ralph-e2e` assertions.

## Tier 1: Connectivity
- Prompt demands exact "PONG" output.
- Assertion checks stdout contains "PONG".

## Tier 2: Orchestration
- **Single iteration**: requires scratchpad updated and `LOOP_COMPLETE` output.
- **Multi-iteration**: expects multiple iterations and events between them.
- Assertions use stdout iteration markers + scratchpad content.

## Tier 3: Events
- **Events scenario**: must emit `<event topic="test.event">...` plus `LOOP_COMPLETE`.
- **Backpressure scenario**: must emit `build.done` with evidence lines: `tests: pass`, `lint: pass`, `typecheck: pass`.

## Tier 4: Capabilities
- Generally assert that outputs contain expected phrases and events; no live tool execution required.

## Tier 5: Hats
- Must emit events to trigger hats (e.g., `build.task` → builder).
- Hat prompts require persona markers (e.g., "Builder role activated").
- Must emit `build.done` or other hat-specific events to allow progression.

## Tier 6: Tasks
- Prompts explicitly instruct running CLI commands like:
  - `ralph task add ...`
  - `ralph task done ...`
- Assertions check:
  - stdout contains task-related text
  - `.agent/tasks.jsonl` exists and is valid
- Mock backend should either execute the exact command or write equivalent files.

## Tier 6: Memories
- Prompts instruct commands like:
  - `ralph tools memory add ...`
  - `ralph tools memory search ...`
- Assertions check:
  - stdout mentions memory activity
  - `.agent/memories.md` exists and contains expected content
- Mock backend should execute the commands or simulate their effects.

## Errors scenarios
- Some tests expect specific exit codes or error text; mock should be able to emit failures deterministically.

## Practical implication for mock adapter
A purely “echo events” mock will fail task/memory/scratchpad scenarios. A cost-free mock should:
- Parse prompts for required outputs and event tags
- Optionally execute **whitelisted local CLI commands** (e.g., `ralph task add`, `ralph tools memory add`) to create files
- Write to `.agent/scratchpad.md` when prompted to update scratchpad
