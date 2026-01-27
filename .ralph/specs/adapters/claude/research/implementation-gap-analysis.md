# Implementation Gap Analysis: Claude Streaming JSON Output

**Date:** 2026-01-14

## Executive Summary

The Claude adapter spec (`claude.spec.md`) describes JSON stream parsing behavior that is **not implemented**. The current implementation captures raw terminal output and looks for XML-style `<event>` tags, but does not use Claude's `--output-format stream-json` flag or parse NDJSON.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Current Architecture                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  CLI Backend          PTY Executor           Event Parser       │
│  ┌──────────┐        ┌─────────────┐        ┌──────────────┐   │
│  │ claude() │───────▶│ run_observe │───────▶│ parse() XML  │   │
│  │          │        │             │        │ <event>      │   │
│  │ NO JSON  │        │ Raw bytes   │        │ tags only    │   │
│  │ flag     │        │ accumulate  │        │              │   │
│  └──────────┘        └─────────────┘        └──────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Current Implementation Status

### ✅ What IS Implemented

| Feature | Location | Notes |
|---------|----------|-------|
| PTY-based execution | `pty_executor.rs:174-226` | Full terminal emulation |
| Raw output capture | `pty_executor.rs:236-356` | Byte accumulation |
| ANSI stripping | `pty_executor.rs:697-705` | For clean text output |
| XML event parsing | `event_parser.rs:26-99` | `<event topic="...">` tags |
| Interactive mode | `pty_executor.rs:373-634` | Bidirectional I/O |
| Idle timeout | `pty_executor.rs` | Activity tracking |
| TUI via EventBus | `main.rs:1202-1230` | Observer pattern |

### ❌ What IS NOT Implemented

| Feature | Spec Reference | Impact |
|---------|---------------|--------|
| `--output-format stream-json` flag | Lines 51, 57 | No structured output |
| NDJSON/framed codec | Lines 80-81 | No line-by-line parsing |
| JSON event extraction | Lines 72-76 | No type-based routing |
| Token-level partial messages | Line 58 | No incremental display |
| Tool invocation metadata | Lines 83-84 | No tool tracking |
| Cost/usage accumulation | Lines 85-86 | No cost tracking |
| Real-time TUI updates | Lines 82 | No streaming display |

## The Exact Gap

### Claude Backend Config (`cli_backend.rs:63-70`)

```rust
pub fn claude() -> Self {
    Self {
        command: "claude".to_string(),
        args: vec!["--dangerously-skip-permissions".to_string()],
        prompt_mode: PromptMode::Stdin,
        prompt_flag: None,
        // MISSING: "--output-format", "stream-json"
    }
}
```

### PTY Executor Output Loop (`pty_executor.rs:236-356`)

```rust
// CURRENT: Raw byte accumulation
let mut output = Vec::new();
loop {
    match reader.read(&mut buf) {
        Ok(n) => {
            output.extend_from_slice(&buf[..n]);
            // MISSING: NDJSON line detection and parsing
        }
    }
}
```

## Claude JSON Stream Format

When invoked with `--output-format stream-json`, Claude emits newline-delimited JSON:

```json
{"type":"system","tools":[...],"model":"claude-opus","session_id":"..."}
{"type":"assistant","message":{"content":[{"type":"text","text":"..."}]},"usage":{...}}
{"type":"assistant","message":{"content":[{"type":"tool_use","id":"...","name":"bash","input":{...}}]}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"...","content":"..."}]}}
{"type":"result","duration_ms":5000,"total_cost_usd":0.02,"num_turns":2,"is_error":false}
```

### Event Types

| Type | Description | Key Fields |
|------|-------------|------------|
| `system` | Session initialization | `tools`, `model`, `session_id` |
| `assistant` | Claude's responses | `message.content` (text, tool_use), `usage` |
| `user` | Tool results | `message.content` (tool_result) |
| `result` | Session complete | `duration_ms`, `total_cost_usd`, `num_turns`, `is_error` |
| `stream_event` | Token deltas | Requires `--verbose --include-partial-messages` |

## Dependencies Available

- `tokio-util::codec::FramedRead` — Already in dependencies
- `serde_json` — Already in dependencies
- Line-by-line parsing can be done manually or with NDJSON crate

## Key Constraints

1. **PTY Still Required** — JSON stream output does not remove Claude's TTY requirement (GitHub #9026)
2. **Stdin Mode** — Recent change to use stdin for prompts (avoids large prompt issues)
3. **Mixed Output** — PTY may include ANSI codes even with JSON format; need to handle gracefully

## Relevant Files

| Component | File | Key Lines |
|-----------|------|-----------|
| Claude backend config | `cli_backend.rs` | 63-70 |
| Command building | `cli_backend.rs` | 143-185 |
| PTY spawning | `pty_executor.rs` | 174-226 |
| Observe mode loop | `pty_executor.rs` | 236-356 |
| Event parsing | `event_parser.rs` | 26-99 |
| Main execution | `main.rs` | 1068-1075 |
