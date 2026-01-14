# Existing Claude Adapter Spec

This document preserves the content from the original `claude.spec.md` for reference during the PDD process.

---

## Configuration

| Property | Value |
|----------|-------|
| Command | `claude` |
| Headless flags | `--dangerously-skip-permissions` |
| Prompt mode | Argument (`-p "prompt"`) |
| TTY required | **Yes** — Ralph auto-enables PTY |
| Auto-detect | `claude --version` |
| Auth | `claude` CLI login (interactive) |

## Invocation

```bash
claude --dangerously-skip-permissions -p "your prompt"
```

## Behavior

### TTY Requirement

Claude hangs indefinitely without a TTY, even with the `-p` flag. This is a known issue ([GitHub #9026](https://github.com/anthropics/claude-code/issues/9026)).

**Ralph's behavior:** When `backend: "claude"` is selected (explicitly or via auto-detection), Ralph auto-enables PTY mode regardless of config. This ensures Claude always has a TTY.

### Large Prompt Handling

Large stdin inputs (>7000 chars) may produce empty output ([GitHub #7263](https://github.com/anthropics/claude-code/issues/7263)).

**Ralph's behavior:** For prompts exceeding the threshold, Ralph writes the prompt to a temp file and instructs Claude to read from it.

### Permission Bypass

The `--dangerously-skip-permissions` flag bypasses all permission prompts. This is required for non-interactive operation — without it, Claude will pause waiting for user approval on file writes, command execution, etc.

## JSON Stream Output

The `--output-format stream-json` flag enables structured NDJSON output, allowing Ralph to parse Claude's output programmatically and forward events to the TUI in real-time.

### Configuration

| Property | Value |
|----------|-------|
| Output format flag | `--output-format stream-json` |
| Partial messages | `--verbose --include-partial-messages` (optional) |

### Invocation

```bash
claude --dangerously-skip-permissions -p "prompt" --output-format stream-json
```

### Stream Event Types

Claude emits newline-delimited JSON objects. Each line is a self-contained event with a `type` field:

| Type | Description | Key Fields |
|------|-------------|------------|
| `system` | Session initialization | `tools`, `model`, `session_id` |
| `assistant` | Claude's responses | `message.content` (text, tool_use), `usage` |
| `user` | Tool results returned to Claude | `message.content` (tool_result) |
| `result` | Session complete | `duration_ms`, `total_cost_usd`, `num_turns`, `is_error` |
| `stream_event` | Token-level deltas | Requires `--verbose --include-partial-messages` |

### Ralph's Behavior (Specified)

**Stream parsing:** Ralph reads Claude's stdout as NDJSON, parsing each line independently using a framed codec. Malformed lines are skipped with a warning rather than crashing the iteration.

**TUI forwarding:** Each parsed event is forwarded to the TUI for real-time display. The TUI renders assistant text incrementally and shows tool invocations as they occur.

**Tool call extraction:** When an `assistant` event contains `tool_use` content, Ralph extracts the tool name and arguments for progress tracking. This enables the TUI to display which tools Claude is invoking.

**Usage accumulation:** Ralph accumulates `usage` stats from each `assistant` event across turns. The final `result` event provides total cost and duration for the iteration.

**Error handling:** When a `result` event has `is_error: true`, Ralph treats the iteration as failed and triggers the appropriate retry or escalation logic.

**PTY requirement:** JSON stream output does not remove Claude's TTY requirement (see GitHub #9026). Ralph must still spawn Claude in a PTY even when using `--output-format stream-json`.

## Acceptance Criteria (Existing)

**Given** `backend: "claude"` in config
**When** Ralph executes an iteration
**Then** PTY mode is auto-enabled regardless of `pty_mode` setting

**Given** `backend: "auto"` and Claude is installed
**When** Ralph starts
**Then** Claude is selected (first in priority order)

**Given** a prompt exceeding 7000 characters
**When** Ralph invokes Claude
**Then** the prompt is written to a temp file to avoid the large stdin bug

### JSON Stream Output

**Given** `output_format: stream-json` is configured
**When** Claude emits events
**Then** Ralph parses each line as JSON and forwards to TUI

**Given** an `assistant` event with `tool_use` content
**When** received by Ralph
**Then** TUI displays tool name and status immediately

**Given** a `result` event with `is_error: true`
**When** received by Ralph
**Then** the iteration is marked as failed

**Given** a malformed JSON line in the stream
**When** parsing fails
**Then** Ralph skips the line and logs a warning without crashing
