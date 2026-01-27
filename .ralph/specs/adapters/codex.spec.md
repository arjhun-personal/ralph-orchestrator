---
status: review
gap_analysis: 2026-01-14
related:
  - ../cli-adapters.spec.md
---

# Codex Adapter

OpenAI's Codex CLI.

## Configuration

| Property | Value |
|----------|-------|
| Command | `codex` |
| Subcommand | `exec` |
| Headless flags | `--full-auto` |
| Prompt mode | Argument (positional) |
| TTY required | No |
| Auto-detect | `codex --version` |
| Auth | `CODEX_API_KEY` environment variable |

## Invocation

```bash
codex exec --full-auto "your prompt"
```

## Behavior

### Subcommand Requirement

Codex requires the `exec` subcommand for non-interactive mode. Running `codex` alone launches an interactive TUI. The subcommand is passed in the adapter's `args` array.

### Full Auto Mode

`--full-auto` is a convenience alias that enables:
- `--sandbox workspace-write` — Sandbox to workspace directory only
- `--ask-for-approval on-request` — Minimal approval prompting

### Output Streams

Progress output goes to stderr; the final response goes to stdout. This separation allows clean piping of results.

## Acceptance Criteria

**Given** `backend: "codex"` in config
**When** Ralph builds the command
**Then** the command includes `exec` subcommand in args array

**Given** `backend: "codex"` in config
**When** Ralph executes an iteration
**Then** `--full-auto` flag is included
