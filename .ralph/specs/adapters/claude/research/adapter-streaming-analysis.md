# Adapter Streaming Output Analysis

**Date:** 2026-01-14

## Executive Summary

Only Claude currently supports structured streaming output. However, the design should be extensible to allow other adapters to adopt streaming in the future without major refactoring.

## Current Adapter Landscape

| Adapter | Structured Output | Status |
|---------|-------------------|--------|
| **Claude** | `--output-format stream-json` | ✅ Ready |
| **Codex** | `--json` (undocumented) | ⚠️ Possible |
| **Kiro** | None documented | ⚠️ Unknown |
| **Gemini** | None documented | ⚠️ Unknown |
| **Amp** | None | ❌ Unlikely |

## Current Architecture Gap

```
┌─────────────┐
│  CliBackend │ ← No output_format field
└──────┬──────┘
       ▼
┌─────────────────┐
│  build_command  │ ← No streaming flag support
└──────┬──────────┘
       ▼
┌─────────────────┐
│  Executor       │ ← Treats all output as raw text
└──────┬──────────┘
       ▼
┌─────────────────┐
│  EventParser    │ ← Only XML tag parsing
└─────────────────┘
```

## Recommended Extensibility Approach

### 1. Add OutputFormat to CliBackend

```rust
pub enum OutputFormat {
    Text,      // Default: raw text, XML event tags
    StreamJson, // NDJSON with structured events
}

pub struct CliBackend {
    pub command: String,
    pub args: Vec<String>,
    pub prompt_mode: PromptMode,
    pub prompt_flag: Option<String>,
    pub output_format: OutputFormat,  // NEW
}
```

### 2. Create OutputParser Trait

```rust
pub trait OutputParser: Send + Sync {
    fn parse_line(&self, line: &str) -> Option<StreamEvent>;
}

// Implementations:
// - TextOutputParser: existing XML tag parsing
// - JsonOutputParser: NDJSON parsing for Claude
```

### 3. Format-Aware Streaming Callbacks

```rust
pub trait StreamHandler {
    fn on_text(&mut self, text: &str);
    fn on_tool_call(&mut self, name: &str, args: &str);
    fn on_tool_result(&mut self, id: &str, output: &str);
    fn on_error(&mut self, error: &str);
    fn on_complete(&mut self, result: &SessionResult);
}
```

## Benefits of This Approach

1. **Backwards Compatible** — Text format remains default; adapters without streaming work unchanged
2. **Opt-In Per Adapter** — Each adapter declares its output format capability
3. **Future-Proof** — When Kiro/Codex add streaming flags, just update their config
4. **Separation of Concerns** — Parsing logic isolated from execution logic

## Claude-Specific Implementation Notes

Claude's NDJSON events:
- `system` → Session init (ignore or log)
- `assistant` → Text content + tool_use blocks
- `user` → Tool results (verbose mode only)
- `result` → Final stats (verbose mode only)

Stream parsing requirements:
- Line-buffered reading (NDJSON = one JSON object per line)
- Skip malformed lines gracefully
- Extract tool names from `tool_use` content blocks
- Accumulate usage stats if verbose

## Files to Modify

| File | Change |
|------|--------|
| `cli_backend.rs` | Add `OutputFormat` enum and field |
| `cli_backend.rs` | Update `claude()` to set format and add flag |
| `pty_executor.rs` | Add line-buffered JSON parsing in read loop |
| New: `stream_parser.rs` | JSON event parser implementation |
| `main.rs` | Route parsed events to output handler |
