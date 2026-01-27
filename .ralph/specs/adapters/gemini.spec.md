---
status: review
gap_analysis: 2026-01-14
related:
  - ../cli-adapters.spec.md
---

# Gemini Adapter

Google's Gemini CLI (`@google/gemini-cli`).

## Configuration

| Property | Value |
|----------|-------|
| Command | `gemini` |
| Headless flags | `--yolo` |
| Prompt mode | Argument (`-p "prompt"`) |
| TTY required | No |
| Auto-detect | `gemini --version` |
| Auth | `GEMINI_API_KEY` environment variable |

## Invocation

```bash
gemini -p "your prompt" --yolo
```

## Behavior

### Auto-Approval

`--yolo` auto-approves all tool actions, equivalent to Claude's `--dangerously-skip-permissions`. Without this flag, Gemini pauses for user confirmation on tool invocations.

### Sandbox

`--sandbox` is enabled by default when using `--yolo`, providing safety boundaries for automated tool execution.

### Headless Operation

No TTY required. Gemini works cleanly in CI/CD pipelines and headless environments without special handling.

## Acceptance Criteria

**Given** `backend: "gemini"` in config
**When** Ralph executes an iteration
**Then** Gemini is invoked with `--yolo` flag

**Given** `backend: "auto"` and Gemini is installed but Claude is not
**When** Ralph starts
**Then** Gemini is selected
