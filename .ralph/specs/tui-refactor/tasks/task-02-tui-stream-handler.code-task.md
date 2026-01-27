---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Create TuiStreamHandler

## Description
Implement `TuiStreamHandler` that implements the `StreamHandler` trait to convert streaming API output into styled `Line<'static>` objects. This handler must produce output that is visually equivalent to `PrettyStreamHandler` but outputs to ratatui Lines instead of the terminal.

## Background
The `StreamHandler` trait defines how streaming content (text, tool calls, tool results, errors) is processed. `PrettyStreamHandler` outputs directly to stdout with colors and formatting. `TuiStreamHandler` must produce the same visual output but store it as ratatui Lines for display in the TUI content pane. Output parity between the two handlers is a key requirement.

## Reference Documentation
**Required:**
- Design: specs/tui-refactor/design/detailed-design.md (Section: Components > TuiStreamHandler)

**Additional References:**
- specs/tui-refactor/context.md (codebase patterns)
- specs/tui-refactor/plan.md (overall strategy)
- `ralph-adapters/src/stream_handler.rs:23-125` — PrettyStreamHandler reference implementation
- `ralph-adapters/src/stream_handler.rs:127-151` — StreamHandler trait definition
- `ralph-adapters/src/stream_handler.rs:231-252` — `format_tool_summary()` utility
- `ralph-adapters/src/stream_handler.rs:258-269` — `truncate()` utility

**Note:** You MUST read the design document and PrettyStreamHandler implementation before beginning.

## Technical Requirements
1. Create `TuiStreamHandler` struct in `ralph-adapters/src/stream_handler.rs`
2. Implement `StreamHandler` trait methods:
   - `on_text(&mut self, text: &str)` — Buffer text, create Lines on newlines
   - `on_tool_call(&mut self, name: &str, id: &str, input: &Value)` — Format as "⚙️ [ToolName] summary"
   - `on_tool_result(&mut self, id: &str, content: &str, is_error: bool)` — Show in verbose, checkmark in quiet
   - `on_error(&mut self, error: &str)` — Red styled error line
3. Buffer partial text until newline (text doesn't always arrive in complete lines)
4. Reuse existing utilities: `format_tool_summary()`, `truncate()`
5. Store output lines via callback or Arc<Mutex<Vec<Line>>>
6. Long lines must be truncated UTF-8 safe (use existing `truncate()` function)

## Dependencies
- Task 1: IterationBuffer (TuiStreamHandler will write to IterationBuffer)

## Implementation Approach
1. **RED**: Write failing tests for each StreamHandler method
2. **GREEN**: Implement TuiStreamHandler with minimal code to pass tests
3. **REFACTOR**: Extract shared formatting utilities if needed, ensure parity with PrettyStreamHandler

## Acceptance Criteria

1. **Text Creates Line**
   - Given TuiStreamHandler
   - When `on_text("hello\n")` is called
   - Then a Line with "hello" content is produced

2. **Partial Text Buffering**
   - Given TuiStreamHandler
   - When `on_text("hel")` then `on_text("lo\n")` is called
   - Then a single "hello" line is produced (after the newline)

3. **Tool Call Format**
   - Given TuiStreamHandler
   - When `on_tool_call("Read", "id", &json!({}))` is called
   - Then a Line starting with "⚙️" and containing "Read" is produced

4. **Tool Result Verbose**
   - Given TuiStreamHandler with verbose=true
   - When `on_tool_result(...)` is called
   - Then result content appears in output

5. **Tool Result Quiet**
   - Given TuiStreamHandler with verbose=false
   - When `on_tool_result(...)` is called
   - Then only checkmark is shown (minimal output)

6. **Error Red Style**
   - Given TuiStreamHandler
   - When `on_error("fail")` is called
   - Then a Line with red foreground style is produced

7. **Text Truncation**
   - Given TuiStreamHandler
   - When `on_text()` receives 500+ character string
   - Then line is truncated with "..." and is UTF-8 safe

8. **Output Parity - Text**
   - Given same text input to both TuiStreamHandler and PrettyStreamHandler
   - When both process the input
   - Then content is equivalent (accounting for style representation differences)

9. **Output Parity - Tool Call**
   - Given same tool call to both handlers
   - When both process the input
   - Then icon and name formatting matches

10. **Output Parity - Error**
    - Given same error to both handlers
    - When both process the input
    - Then both show red styled error

11. **Unit Tests Pass**
    - Given the implementation is complete
    - When running `cargo test -p ralph-adapters tui_stream`
    - Then all tests pass

## Metadata
- **Complexity**: High
- **Labels**: foundation, stream-handler, adapters
- **Required Skills**: Rust, StreamHandler trait, ratatui styling, text buffering
