# Mock Adapter E2E Plan Summary

## Artifacts Created
- `specs/mock-adapter-e2e/rough-idea.md`
- `specs/mock-adapter-e2e/idea-honing.md`
- `specs/mock-adapter-e2e/research/event-loop-and-headless.md`
- `specs/mock-adapter-e2e/research/backend-cli-patterns.md`
- `specs/mock-adapter-e2e/research/e2e-harness.md`
- `specs/mock-adapter-e2e/research/cost-free-e2e-strategy.md`
- `specs/mock-adapter-e2e/research/e2e-scenario-needs.md`
- `specs/mock-adapter-e2e/design/detailed-design.md`
- `specs/mock-adapter-e2e/implementation/plan.md`

## Design Overview
The plan introduces a cost‑free E2E mode that reuses the existing `ralph run` PTY execution path while replacing paid backends with a mock CLI. A new `ralph-e2e mock-cli` subcommand replays SessionRecorder JSONL cassettes and can execute a strict whitelist of local commands for side‑effects (tasks/memories). The `ralph-e2e --mock` flag writes a custom backend config for each scenario and bypasses backend availability/auth checks. Cassette naming is deterministic with fail‑fast missing behavior and accelerated replay by default.

## Implementation Overview
The implementation plan is staged to:
1) Add mock flags + cassette resolution,
2) Add the mock CLI subcommand,
3) Wire mock mode into scenario setup and runner,
4) Add whitelist command execution,
5) Add tests and initial cassettes.

## Suggested Next Steps
1. Review the detailed design at `specs/mock-adapter-e2e/design/detailed-design.md`.
2. Review the implementation plan at `specs/mock-adapter-e2e/implementation/plan.md`.
3. If approved, begin implementation with Step 1.

## Areas to Refine (if needed)
- The exact whitelist command grammar and timeouts.
- How to represent error/timeout behavior in cassettes.

