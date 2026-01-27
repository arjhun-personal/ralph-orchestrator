---
status: review
gap_analysis: 2026-01-14
related:
  - ../cli-adapters.spec.md
---

# Amp Adapter

Sourcegraph's Amp CLI (`@sourcegraph/amp`).

## Configuration

| Property | Value |
|----------|-------|
| Command | `amp` |
| Headless flags | `--dangerously-allow-all -x` |
| Prompt mode | Argument (`-x "prompt"`) |
| TTY required | No |
| Auto-detect | `amp --version` |
| Auth | `AMP_API_KEY` environment variable |

## Invocation

```bash
amp --dangerously-allow-all -x "your prompt"
```

## Behavior

### Execute Mode

`-x` / `--execute` enables execute mode: send prompt, wait for completion, print response, exit. Without this flag, Amp enters interactive mode.

### Auto-Approval

`--dangerously-allow-all` auto-approves all tool invocations. Required for full automation — without it, Amp pauses for user confirmation.

### Research Preview

Amp is in research preview. Expect rough edges and potential breaking changes. Thread storage is cloud-based (ampcode.com/threads).

## Acceptance Criteria

**Given** `backend: "amp"` in config
**When** Ralph executes an iteration
**Then** both `--dangerously-allow-all` and `-x` flags are included

**Given** `backend: "amp"` in config
**When** Ralph builds the command
**Then** the prompt is passed as an argument to `-x`, not via stdin
