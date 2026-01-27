
## Question 1: Default vs Opt-In Mocking
Should the cost-free mock adapter be the **default** path for `ralph-e2e`, or an **opt-in** mode (e.g., a flag like `--mock` / `--backend mock`)?
**Answer:** Opt-in via a flag (e.g., `--mock` / `--backend mock`).

## Question 2: Mock adapter implementation location
Which approach should we use for the mock adapter implementation?
- A. Separate mock CLI binary/script invoked via `cli.backend: custom`
- B. Built-in `mock` backend in ralph-adapters
- C. Replay backend using JSONL cassettes directly

**Answer:** A. Separate mock CLI binary/script invoked via `cli.backend: custom`.

## Question 3: Mock fidelity level
What fidelity should the mock provide?
- Minimal: echo required keywords/events
- Moderate: parse prompt + simulate behavior
- High: replay recorded cassettes with timing + tool output simulation

**Answer:** High fidelity (replay recorded cassettes with timing + tool output simulation).

## Question 4: Mock output source of truth
What should be the source of truth for mock outputs?
- A. JSONL cassettes (SessionRecorder format)
- B. YAML/JSON response scripts (custom)
- C. Prompt-driven rules

**Answer:** A. JSONL cassettes (SessionRecorder format).

## Question 5: Cassette selection
How should mock mode choose which cassette to replay for a given E2E scenario?
**Answer:** Deterministic naming with fallback: `cassettes/e2e/<scenario-id>-<backend>.jsonl` else `cassettes/e2e/<scenario-id>.jsonl`. Fail fast if missing.

## Question 6: Tool-use side effects in mock mode
How should the mock CLI handle tool-use side effects (tasks/memories)?
- A. Execute a whitelist of local commands (e.g., `ralph task add`, `ralph tools memory add`)
- B. Simulate effects by writing files directly
- C. Ignore side effects and relax E2E assertions

**Answer:** A. Execute a strict whitelist of local commands with guardrails.

## Question 7: Replay timing
Should mock replay **honor recorded timing** (realistic delays) or **run instantly/accelerated** by default?
**Answer:** Accelerated by default.

## Question 8: Mock CLI packaging
Where should the mock CLI live, and in what form?
**Answer:** Use `ralph-e2e` as the mock CLI via a new subcommand (e.g., `ralph-e2e mock-cli`).

## Question 9: Backend availability/auth checks in mock mode
Should `ralph-e2e --mock` bypass backend availability/auth checks and run regardless of real CLI installation/auth?
**Answer:** Yes, bypass availability/auth checks in mock mode.

## Question 10: Mock run matrix
In mock mode, should we run each scenario once per backend (Claude/Kiro/OpenCode) to preserve the existing report matrix, or run each scenario only once under a single "Mock" backend label?
**Answer:** Preserve the per-backend matrix (run each scenario per backend) in mock mode.

## Question 11: Missing cassette behavior
If a cassette is missing for a scenario/backend in mock mode, should the run:
- Fail fast for that scenario
- Skip the scenario with a warning
- Fall back to prompt-driven mock output
**Answer:** Fail fast if a cassette is missing.

## Question 12: Iteration markers in mock output
Since `ralph` prints iteration separators in non-TUI mode, should the mock CLI **avoid** injecting iteration markers and just replay cassette output as-is?
**Answer:** Do not inject iteration markers; mock CLI should replay output from real adapters as-is.

## Question 13: Cassette directory selection
Should mock mode support multiple cassette directories or just a single default location?

**Answer:** Single default location.

## Question 14: Error/timeout scenarios in mock mode
Should mock mode run the **error-handling scenarios** (timeout, max-iterations, auth-failure, backend-unavailable) using dedicated cassettes and behavior controls (e.g., simulated delay/exit codes), or should those be skipped in mock mode?
**Answer:** Yes, run error-handling scenarios in mock mode using dedicated cassettes/controls.

## Question 15: Mock CLI interface
What should the `ralph-e2e mock-cli` interface look like (flags/env), at minimum?
**Answer:** Minimum interface: `--cassette <path>`, `--speed <n>`, and a whitelist mechanism for allowed local commands (flag or env).

## Question 16: Are requirements clarification complete?
Do you feel the requirements clarification is complete, or do you want to add/adjust anything before we move to design?
**Answer:** Requirements clarification complete.
