---
status: completed
created: 2026-01-15
started: 2026-01-15
completed: 2026-01-15
---
# Task: Enhance Headless Tool Output with Arguments

## Description
Improve the Claude adapter's headless output to display tool arguments (file paths, commands, patterns) alongside tool names. Currently the output shows only `[Tool] Read` but should show `[Tool] Read: src/main.rs` to provide meaningful context during headless execution.

## Background
When running Ralph in headless mode, the console output for tool calls is minimal—showing only the tool name without any context about what file is being read, what command is being executed, or what pattern is being searched. This makes it difficult to follow agent progress without enabling verbose mode.

The tool input data is already available in the parsed `ContentBlock::ToolUse` structure (with `id`, `name`, and `input` fields), but the `input` field is currently being discarded during event dispatch at `pty_executor.rs:1268` using the `..` pattern match syntax.

## Technical Requirements
1. Update `StreamHandler` trait's `on_tool_call` method signature to accept tool input as `&serde_json::Value`
2. Implement a smart formatting helper that extracts the most relevant field per tool type
3. Update `ConsoleStreamHandler` to format and display tool arguments
4. Update `QuietStreamHandler` to accept the new signature (no output change needed)
5. Update `dispatch_stream_event()` in `pty_executor.rs` to pass the `input` field
6. Ensure backwards compatibility with any other `StreamHandler` implementations

## Dependencies
- `serde_json` crate (already a dependency)
- Understanding of Claude CLI tool call JSON structure

## Implementation Approach

### Step 1: Update StreamHandler Trait
In `crates/ralph-adapters/src/stream_handler.rs`, modify the trait signature:

```rust
// Current (line 26):
fn on_tool_call(&mut self, name: &str, id: &str);

// New:
fn on_tool_call(&mut self, name: &str, id: &str, input: &serde_json::Value);
```

### Step 2: Add Smart Formatting Helper
Create a helper function that extracts the most relevant field based on tool type:

```rust
fn format_tool_summary(name: &str, input: &serde_json::Value) -> Option<String> {
    let value = match name {
        "Read" | "Edit" | "Write" => input.get("file_path")?.as_str()?,
        "Bash" => {
            let cmd = input.get("command")?.as_str()?;
            return Some(truncate(cmd, 60).to_string());
        }
        "Grep" => input.get("pattern")?.as_str()?,
        "Glob" => input.get("pattern")?.as_str()?,
        "Task" => input.get("description")?.as_str()?,
        "WebFetch" => input.get("url")?.as_str()?,
        _ => return None,
    };
    Some(value.to_string())
}
```

### Step 3: Update ConsoleStreamHandler
Modify the `on_tool_call` implementation to use the formatter:

```rust
fn on_tool_call(&mut self, name: &str, _id: &str, input: &serde_json::Value) {
    match format_tool_summary(name, input) {
        Some(summary) => writeln!(self.stdout, "[Tool] {}: {}", name, summary),
        None => writeln!(self.stdout, "[Tool] {}", name),
    };
}
```

### Step 4: Update QuietStreamHandler
Update signature to match trait (implementation remains empty):

```rust
fn on_tool_call(&mut self, _name: &str, _id: &str, _input: &serde_json::Value) {}
```

### Step 5: Wire Up Dispatch Code
In `crates/ralph-adapters/src/pty_executor.rs` at line 1268, update the pattern match:

```rust
// Current:
ContentBlock::ToolUse { name, id, .. } => handler.on_tool_call(&name, &id),

// New:
ContentBlock::ToolUse { name, id, input } => handler.on_tool_call(&name, &id, &input),
```

## Acceptance Criteria

1. **Read Tool Shows File Path**
   - Given a Claude session that invokes the Read tool
   - When the headless output is displayed
   - Then output shows `[Tool] Read: {file_path}` instead of just `[Tool] Read`

2. **Bash Tool Shows Command (Truncated)**
   - Given a Claude session that invokes the Bash tool with a long command
   - When the headless output is displayed
   - Then output shows `[Tool] Bash: {command}` truncated to ~60 characters

3. **Edit Tool Shows File Path**
   - Given a Claude session that invokes the Edit tool
   - When the headless output is displayed
   - Then output shows `[Tool] Edit: {file_path}`

4. **Grep Tool Shows Pattern**
   - Given a Claude session that invokes the Grep tool
   - When the headless output is displayed
   - Then output shows `[Tool] Grep: {pattern}`

5. **Unknown Tools Graceful Fallback**
   - Given a Claude session that invokes an unknown or new tool type
   - When the headless output is displayed
   - Then output shows `[Tool] {name}` without crashing (graceful fallback)

6. **Quiet Mode Unchanged**
   - Given Ralph running in quiet mode
   - When tools are invoked
   - Then no tool output is displayed (behavior unchanged)

7. **Build and Tests Pass**
   - Given the implementation is complete
   - When running `cargo build` and `cargo test`
   - Then all builds succeed and all tests pass

8. **Unit Tests for Formatter**
   - Given the format_tool_summary helper function
   - When running the test suite
   - Then unit tests cover all supported tool types and edge cases

## Example Output Transformation

**Before:**
```
[Tool] Read
[Tool] Read
[Tool] Read
[Tool] Bash
[Tool] Edit
[Tool] Grep
```

**After:**
```
[Tool] Read: src/stream_handler.rs
[Tool] Read: src/pty_executor.rs
[Tool] Read: Cargo.toml
[Tool] Bash: cargo test --lib
[Tool] Edit: src/main.rs
[Tool] Grep: StreamHandler
```

## Metadata
- **Complexity**: Medium
- **Labels**: Enhancement, CLI, UX, Streaming, Adapter
- **Required Skills**: Rust, trait design, JSON handling, pattern matching
