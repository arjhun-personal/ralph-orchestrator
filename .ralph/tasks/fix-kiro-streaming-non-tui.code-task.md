---
status: completed
created: 2026-01-20
started: 2026-01-20
completed: 2026-01-20
---
# Task: Fix Kiro CLI Streaming in Non-TUI Mode

## Description
Fix the regression where Kiro CLI output no longer streams progressively when running without `--tui`. The `PrettyStreamHandler` buffers text but never flushes until `on_complete()` or `on_tool_call()` is called. For Text format backends like Kiro (which don't emit JSON events), text should flush immediately in `on_text()` to show streaming output progressively.

## Background
Kiro uses `OutputFormat::Text` (raw text streaming) while Claude uses `OutputFormat::StreamJson` (NDJSON with structured events). In non-TUI mode, `PrettyStreamHandler` is used, and its `on_text()` method only buffers text without flushing. The buffer only flushes when:
- `on_tool_call()` is called (which requires JSON events that Kiro doesn't emit)
- `on_complete()` is called (at session end)

This causes all Kiro output to appear at once when the session completes, rather than streaming progressively.

In TUI mode, `TuiStreamHandler` immediately updates shared state on every `on_text()` call, which is why TUI mode works correctly.

## Technical Requirements
1. Modify `PrettyStreamHandler.on_text()` to flush the text buffer immediately after appending
2. Ensure markdown rendering still works correctly with incremental flushes
3. Maintain backward compatibility with Claude's StreamJson format (which may send partial markdown)
4. Keep the existing buffer mechanism for potential future use cases where batching is needed

## Dependencies
- File: `crates/ralph-adapters/src/stream_handler.rs`
- `PrettyStreamHandler` struct (lines 37-69)
- `on_text()` method (lines 72-75)
- `flush_text_buffer()` method (lines 58-68)

## Implementation Approach
1. In `PrettyStreamHandler.on_text()`, add a call to `flush_text_buffer()` after appending text
2. This ensures each chunk of text is rendered and displayed immediately
3. The markdown rendering via `termimad` should handle incremental text reasonably well

## Acceptance Criteria

1. **Immediate Text Display**
   - Given Kiro CLI running in non-TUI mode
   - When text output is received from the CLI
   - Then text appears immediately in the terminal (not buffered until completion)

2. **Streaming Behavior**
   - Given a long-running Kiro session with progressive output
   - When output is being generated
   - Then each chunk appears as it arrives, not all at once at the end

3. **Claude Compatibility**
   - Given Claude CLI running in non-TUI mode
   - When text and tool call events are received
   - Then output still renders correctly with markdown formatting

4. **Smoke Tests Pass**
   - Given the modified code
   - When running `cargo test -p ralph-core smoke_runner`
   - Then all smoke tests pass

## Metadata
- **Complexity**: Low
- **Labels**: Bug Fix, Streaming, Kiro, UX
- **Required Skills**: Rust, Stream handling
