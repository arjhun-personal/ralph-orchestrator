---
status: review
gap_analysis: 2026-01-14
related:
  - ../cli-adapters.spec.md
  - ../interactive-mode.spec.md
---

# Kiro Adapter

AWS's Kiro CLI coding assistant (formerly Amazon Q Developer CLI).

## Configuration

| Property | Value |
|----------|-------|
| Command | `kiro-cli` |
| Subcommand | `chat` |
| Prompt mode | Argument (positional after flags) |
| TTY required | No |
| Auto-detect | `kiro-cli --version` |
| Auth | `kiro-cli` login (interactive) |

## Flags by Mode

| Mode | Flags |
|------|-------|
| **Autonomous** | `--no-interactive --trust-all-tools` |
| **Interactive** | `--trust-all-tools` |

The `--no-interactive` flag disables confirmation prompts and causes kiro-cli to exit on Ctrl+C. In interactive mode, this flag is omitted so users can respond to prompts and use Ctrl+C to cancel operations without terminating the session.

## Invocation

### Autonomous Mode (default)

```bash
kiro-cli chat --no-interactive --trust-all-tools "your prompt"
```

### TUI Mode (`ralph run --tui`)

```bash
kiro-cli chat --trust-all-tools "your prompt"
```

## Behavior

### Subcommand Requirement

Kiro requires the `chat` subcommand for prompt execution. The subcommand is passed in the adapter's `args` array, not as part of the command.

### Tool Trust

`--trust-all-tools` enables autonomous tool use without confirmation prompts. Built-in tools include: `read`, `write`, `shell`, `aws`, `report`. This flag is used in both modes.

### Ctrl+C Handling

| Mode | Ctrl+C Behavior |
|------|-----------------|
| **Autonomous** (`--no-interactive`) | Kiro exits immediately |
| **Interactive** (no `--no-interactive`) | Kiro may prompt for confirmation or cancel current operation |

This is why interactive mode omits `--no-interactive`—it allows Ralph's double Ctrl+C logic to work as intended.

### Known Issue

In autonomous mode, `--no-interactive` may occasionally still prompt for input. A fix is in progress upstream. Ralph handles unexpected prompts by timing out.

## Acceptance Criteria

**Given** `backend: "kiro"` in config
**When** Ralph builds the command
**Then** the command includes `chat` subcommand in args array

**Given** `backend: "kiro"` in autonomous mode
**When** Ralph builds the command
**Then** args include `--no-interactive --trust-all-tools`

**Given** `backend: "kiro"` in interactive mode
**When** Ralph builds the command
**Then** args include `--trust-all-tools` but NOT `--no-interactive`

**Given** `backend: "kiro"` in interactive mode
**When** user presses Ctrl+C once
**Then** Ctrl+C is forwarded to kiro and kiro does NOT exit immediately
