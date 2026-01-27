---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Add Markdown Rendering to TUI Mode

## Description
Add markdown rendering support to TUI mode so that Claude's output displays with proper formatting (bold, italic, code blocks, headers, lists) instead of raw markdown syntax. Currently, `PrettyStreamHandler` uses `termimad` for beautiful terminal markdown rendering, but `TuiStreamHandler` passes raw text directly to ratatui `Line` objects, resulting in users seeing `**bold**` instead of **bold**.

## Background
The TUI mode uses a different rendering pipeline than the standard CLI:
- **Non-TUI**: `PrettyStreamHandler` → `termimad::MadSkin` → ANSI codes → stdout
- **TUI**: `TuiStreamHandler` → raw `Line<'static>` objects → `ContentPane` widget → ratatui

The `tui-markdown` crate (by @joshka, a ratatui maintainer) bridges this gap by converting markdown to ratatui `Text` objects using `pulldown-cmark` under the hood.

**Key architectural consideration:** Text arrives in streaming chunks via `on_text()` calls. The recommended approach is to accumulate text in a buffer and re-parse the entire buffer on each update—pulldown-cmark is fast enough for this pattern.

## Technical Requirements
1. Add `tui-markdown` crate dependency to `ralph-adapters`
2. Create a markdown-to-Lines conversion function that transforms `tui_markdown::from_str()` output into `Vec<Line<'static>>`
3. Modify `TuiStreamHandler::on_text()` to parse accumulated text through markdown conversion
4. Preserve existing styling for tool calls (blue gear emoji), tool results (gray checkmark), and errors (red X)
5. Handle streaming edge cases: incomplete markdown blocks, unclosed formatting, code fences spanning chunks
6. Ensure the ContentPane widget correctly renders the styled Lines (may need minor adjustments)

## Dependencies
- `tui-markdown` crate (uses `pulldown-cmark` internally)
- Existing ratatui types: `Line`, `Span`, `Style`, `Color`, `Modifier`
- Current `TuiStreamHandler` implementation in `crates/ralph-adapters/src/stream_handler.rs`
- `ContentPane` widget in `crates/ralph-tui/src/widgets/content.rs`

## Implementation Approach

### Phase 1: Add Dependency and Basic Integration
1. Add `tui-markdown = "0.3"` to `crates/ralph-adapters/Cargo.toml`
2. Create helper function to convert `tui_markdown::Text` → `Vec<Line<'static>>`
3. Add a `markdown_buffer: String` field to `TuiStreamHandler` for full-text accumulation

### Phase 2: Modify Streaming Handler
1. In `on_text()`, append to `markdown_buffer` instead of processing character-by-character for plain text
2. After each append, re-parse the full buffer with `tui_markdown::from_str()`
3. Replace the lines in the shared `Arc<Mutex<Vec<Line>>>` with the parsed result
4. Keep the existing character-by-character newline detection for determining when to trigger re-parse

### Phase 3: Preserve Existing Styling
1. Tool calls (`on_tool_call`), tool results (`on_tool_result`), and errors should bypass markdown parsing
2. These already have explicit styling (blue, gray, red) that should be preserved
3. Only Claude's text output should go through markdown rendering

### Phase 4: Handle Edge Cases
1. **Incomplete bold/italic**: `**partial` should render as plain text until closed
2. **Unclosed code fences**: Accumulate until fence is closed, render as plain text meanwhile
3. **Headers at buffer end**: `# Title` without newline should render as header
4. Consider adding a "finalize" method called on stream end to flush any pending partial blocks

## Acceptance Criteria

1. **Bold Text Rendering**
   - Given Claude outputs `**important**` in TUI mode
   - When the text is displayed in ContentPane
   - Then "important" appears with bold styling (Modifier::BOLD)

2. **Italic Text Rendering**
   - Given Claude outputs `*emphasized*` in TUI mode
   - When the text is displayed in ContentPane
   - Then "emphasized" appears with italic styling (Modifier::ITALIC)

3. **Inline Code Rendering**
   - Given Claude outputs `` `code` `` in TUI mode
   - When the text is displayed in ContentPane
   - Then "code" appears with distinct styling (typically colored background or different color)

4. **Code Block Rendering**
   - Given Claude outputs a fenced code block with triple backticks
   - When the text is displayed in ContentPane
   - Then the code block content appears with code styling, preserving indentation

5. **Header Rendering**
   - Given Claude outputs `## Section Title`
   - When the text is displayed in ContentPane
   - Then "Section Title" appears with header styling (bold, possibly colored)

6. **Tool Call Styling Preserved**
   - Given a tool call event occurs during TUI streaming
   - When the tool call is displayed
   - Then it still shows the blue gear emoji and blue-colored tool name

7. **Error Styling Preserved**
   - Given an error event occurs during TUI streaming
   - When the error is displayed
   - Then it still shows the red X emoji and red-colored error message

8. **Streaming Continuity**
   - Given markdown text arrives in multiple chunks (e.g., `**bo` then `ld**`)
   - When all chunks have been received
   - Then the complete markdown renders correctly as bold "bold"

9. **Partial Markdown Graceful Handling**
   - Given incomplete markdown at stream end (e.g., unclosed `**`)
   - When the stream finalizes
   - Then the partial text renders as plain text without crashing

10. **Performance Acceptable**
    - Given continuous streaming of markdown content
    - When re-parsing on each chunk
    - Then the TUI remains responsive with no visible lag

## Files to Modify

- `crates/ralph-adapters/Cargo.toml` - Add tui-markdown dependency
- `crates/ralph-adapters/src/stream_handler.rs` - Modify TuiStreamHandler
- `crates/ralph-tui/src/widgets/content.rs` - Adjust if needed for styled content

## Testing Strategy

1. **Unit tests** for markdown-to-Lines conversion function
2. **Integration test** using existing smoke test infrastructure with markdown-heavy fixture
3. **Manual validation** with `/tui-validate` skill for visual verification

## Metadata
- **Complexity**: Medium
- **Labels**: TUI, Markdown, Rendering, Streaming, UX
- **Required Skills**: Rust, ratatui, streaming architecture, markdown parsing
