# Requirements Clarification

This document captures the Q&A process for refining the Claude adapter streaming output feature.

---

## Q1: What should users see when running `ralph run -P PROMPT.md`?

Currently, users see nothing until Claude completes. What output would be most valuable during execution?

**Answer:** Two-tier output verbosity:
- **Default mode:** Assistant text and tool invocations
- **Verbose mode:** Everything (assistant text, tool invocations, tool results, progress indicators, usage stats)

---

## Q2: How should the output be formatted?

For non-interactive terminal output, we have several formatting choices:

**Answer:** Plain text streaming format:
```
Claude: I'll start by reading the file...
[Tool] Read: src/main.rs
Claude: Now I'll make the changes...
[Tool] Edit: src/main.rs
```

---

## Q3: How should verbose mode be enabled?

**Answer:** Introduce verbosity flag if it doesn't exist. Use idiomatic precedence:
1. CLI flag (`--verbose` / `-v`) — highest priority
2. Environment variable (`RALPH_VERBOSE`)
3. Config file (`verbose: true`) — lowest priority

---

## Q4: How should errors and failures be displayed?

**Answer:** Inline errors with stderr separation:
- Errors appear inline in the stream for real-time context
- Errors also written to stderr for Unix-idiomatic behavior
- Enables `2>errors.log` for error logging and CI/CD integration

Example:
```
Claude: Let me run the build...
[Tool] Bash: cargo build
[Error] Build failed: 3 type errors        ← also written to stderr
Claude: I see the errors, let me fix them...
```

---

## Q5: What is the scope of this feature?

**Answer:** Narrowly scoped:
- **Non-interactive only:** Only affects `ralph run` command
- **Claude-only:** Only the Claude adapter; other backends unchanged
- TUI/interactive mode remains unchanged

---

## Q6: Should a summary be displayed at the end?

**Answer:** Verbose only — summary (duration, cost, turns) shown only in verbose mode.

---

## Q7: How should malformed JSON lines be handled?

**Answer:** Skip silently with debug-level logging:
- **Default:** Skip malformed lines silently, continue processing
- **Debug logging:** Log skipped lines at DEBUG/TRACE level for troubleshooting
- Keeps output clean while preserving diagnostics for debugging

---

## Q8: Is streaming output always-on or opt-in?

**Answer:** Always-on with opt-out:
- Streaming output is the default behavior for `ralph run` with Claude backend
- `--quiet` flag (or similar) to suppress streaming for CI/scripting use cases

---

## Requirements Clarification Complete ✓

*Completed: 2026-01-14*

