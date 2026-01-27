---
status: current
gap_analysis: 2026-01-15
related:
  - feature-parity.spec.md
---

# Feature Parity Spec: v1 → v2 Migration

## Context

Ralph Orchestrator v1 (Python) accumulated significant feature bloat that contradicts the elegance of the [Ralph Wiggum pattern](https://github.com/ghuntley/how-to-ralph-wiggum). This spec applies **KISS** and **YAGNI** principles to define which v1 features belong in v2.

### The Ralph Wiggum Philosophy

> "The continuation mechanism is elegantly simple" — Ralph loops should be **simple bash loops** that feed prompts to agents. Complexity belongs in the agent, not the orchestrator.

**Core principles:**
- Fresh context window per iteration
- Persistent state via `.agent/scratchpad.md`
- Backpressure through tests, builds, lints
- Completion signaling for early exit
- Human role: environment engineering, not micromanagement

---

## Feature Triage

### ✅ KEEP — Essential for Ralph Pattern

| Feature | v1 Location | v2 Status | Notes |
|---------|-------------|-----------|-------|
| **Core Loop** | `orchestrator.py` | ✅ Done | Event loop with iteration tracking |
| **Completion Promise** | `orchestrator.py` | ✅ Done | Early exit when agent signals done |
| **Max Iterations** | `safety.py` | ✅ Done | Safety guardrail |
| **Max Runtime** | `safety.py` | ✅ Done | Safety guardrail |
| **Max Cost** | `safety.py` | ✅ Done | Safety guardrail (optional) |
| **Consecutive Failure Limit** | `safety.py` | ✅ Done | Prevent infinite error loops |
| **Claude Backend** | `adapters/claude.py` | ✅ Done | Primary agent |
| **Gemini Backend** | `adapters/gemini.py` | ✅ Done | Alternative agent |
| **Config File (YAML)** | `main.py` | ✅ Done | v1 + v2 format support |
| **Auto-Backend Detection** | `orchestrator.py` | ✅ Done | `backend: auto` |
| **Instruction Builder** | N/A (new in v2) | ✅ Done | Meta-prompt injection |
| **Multi-Hat Mode** | N/A (new in v2) | ✅ Done | Event-driven multi-agent |
| **Kiro Backend** | `adapters/kiro.py` | ✅ Done | AWS Kiro CLI adapter |
| **TUI (Terminal UI)** | N/A (new in v2) | ✅ Done | Full ratatui-based interface |

### ⚠️ SIMPLIFY — Keep Concept, Reduce Scope

| v1 Feature | v1 Complexity | v2 Approach |
|------------|---------------|-------------|
| **Output Formatting** | 4 formatters (Rich, JSON, Plain, Console) with content detection, syntax highlighting, emoji support | **Single formatter**: plain text with optional color. Agents produce readable output. |
| **Logging** | 3 loggers (RalphLogger, AsyncFileLogger, VerboseLogger) with rotation, Unicode sanitization | **Single tracing subscriber**: stdout + optional file. Let shell redirect handle the rest. |
| **Metrics** | CostTracker, iteration history, trigger reasons, JSON export | **Minimal tracking**: iteration count, elapsed time, cumulative cost. Print summary at end. |
| **Signal Handling** | Complex graceful shutdown with subprocess cleanup | **Simple**: Ctrl-C kills process. Git commits provide recovery points. |

### ❌ CUT — YAGNI / Over-Engineering

| v1 Feature | Location | Reason to Cut |
|------------|----------|---------------|
| **Web Dashboard** | `web/` (FastAPI, WebSocket, Chart.js) | Against Ralph philosophy. Watch terminal or `tail -f` logs. |
| **JWT Authentication** | `web/auth.py` | No web dashboard = no auth needed |
| **SQLite Database** | `web/database.py` | History lives in git commits |
| **ACP Protocol** | `adapters/acp*.py` | Over-abstracted. Just run CLI commands. |
| **QChat Adapter** | `adapters/qchat.py` | Deprecated in v1, don't port |
| **Fuzzy Loop Detection** | `safety.py` (rapidfuzz) | Agents self-correct. Consecutive failures sufficient. |
| **Context Optimization** | `context.py` | CLI tools manage their context. Not our job. |
| **Prompt Summarization** | `context.py` | Same as above |
| **Security Masking** | `security.py` | CLI tools (Claude) handle sensitive data |
| **Path Sanitization** | `security.py` | Agent sandboxing handles this |
| **Tool Call Tracking** | `output/base.py` | Agents report their own tool usage |
| **Per-Tool Pricing Tables** | `metrics.py` | CLI tools report costs. Don't duplicate. |
| **Async Execution Mode** | `adapters/base.py` | Single agent at a time. Sync is fine. |
| **MCP Server Inheritance** | `adapters/claude.py` | Claude handles via `--inherit-user-settings` |
| **Tool Allowlist/Denylist** | `adapters/claude.py` | Managed by CLI tools, not orchestrator |
| **Archive Prompts** | `orchestrator.py` | Git history provides this |
| **Rate Limiting** | `web/rate_limit.py` | No web API = no rate limiting |

---

## v2 Feature Completeness Checklist

### Core Orchestration
- [x] Event loop with iteration management
- [x] Completion promise detection
- [x] Safety limits (iterations, runtime, cost, failures)
- [x] Single-hat mode (autonomous loop)
- [x] Multi-hat mode (event-driven multi-agent)

### Configuration
- [x] YAML config file support
- [x] v1 flat format backward compatibility
- [x] v2 nested format
- [x] CLI argument overrides
- [x] Validation with warnings for deprecated fields

### Backend Adapters
- [x] Claude (`claude` CLI)
- [x] Gemini (`gemini` CLI)
- [x] Codex (`codex` CLI)
- [x] Amp (`amp` CLI)
- [x] Kiro (`kiro-cli` CLI)
- [x] Custom (user-defined command)
- [x] Auto-detection with priority ordering

### Instruction System
- [x] Phase-based workflow (orient → select → implement → validate → commit)
- [x] Scratchpad authority
- [x] Backpressure enforcement
- [x] Completion signaling

### Output & Feedback
- [x] Basic stdout logging (info level)
- [x] Optional verbose mode (debug level)
- [x] Termination summary (reason, iterations, time, cost)
- [x] Colored output (optional, via `--color` flag)
- [x] TUI mode (`-i` flag with ratatui-based interface)

### Deferred (Implement If Requested)
- [ ] Prompt file archiving
- [ ] Metrics export (JSON/CSV)

---

## Migration Notes

### For v1 Users

**Config changes:**
- `agent` → `cli.backend` (auto-mapped)
- `max_runtime` → `event_loop.max_runtime_seconds` (auto-mapped)
- `tool_permissions` → Removed (use CLI tool flags)
- `retry_delay` → Removed (not needed with fresh contexts)

**Removed features:**
- Web dashboard: Use `tail -f` on logs or watch terminal
- Rich formatting: Plain output, agents format their own responses
- Cost tracking: CLI tools report costs in their output

**New features:**
- Multi-hat mode for multi-agent orchestration
- Event-driven communication between agents
- Instruction injection with phase-based workflow

### Why Less is More

The original Ralph Wiggum technique uses a **30-line bash script**. v1 grew to **~5,000 lines of Python** with:
- 5 adapters (2 deprecated)
- 4 output formatters
- 3 logging systems
- Web dashboard with auth
- Database for history

v2 targets **~3,000 lines of Rust** with:
- Focused feature set
- Type-safe event system
- No runtime dependencies beyond CLI tools
- Single binary distribution

> "Perfection is achieved not when there is nothing more to add, but when there is nothing left to take away." — Antoine de Saint-Exupéry

---

## Acceptance Criteria

### Given v1 config file
- **When** user runs `ralph` with v1 flat format config
- **Then** orchestrator normalizes to v2 format and executes

### Given completion promise in output
- **When** agent outputs the completion promise string
- **Then** loop terminates with success status

### Given safety limit reached
- **When** iterations, runtime, or cost exceeds configured limit
- **Then** loop terminates with appropriate reason in summary

### Given multi-hat config
- **When** config defines multiple hats with subscriptions
- **Then** orchestrator routes events between hats correctly

### Given no backend specified
- **When** `cli.backend` is "auto" or omitted
- **Then** orchestrator auto-detects first available CLI tool
