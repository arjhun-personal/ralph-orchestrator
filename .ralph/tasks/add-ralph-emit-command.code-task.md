---
status: completed
created: 2026-01-15
started: 2026-01-15
completed: 2026-01-15
---
# Task: Add `ralph emit` CLI Command for Deterministic Event Writing

## Description
Add a `ralph emit` subcommand that allows agents to emit events to `.agent/events.jsonl` with guaranteed valid JSON formatting. This eliminates the error-prone manual `echo` approach where agents write malformed JSONL due to unescaped newlines in payloads.

## Background
Agents currently write events via raw bash commands:
```bash
echo '{"topic":"build.done","payload":"...","ts":"..."}' >> .agent/events.jsonl
```

This fails when payloads contain newlines (especially YAML-structured data), producing broken JSONL. A helper command that handles JSON serialization deterministically solves this at the source.

The ralph-cli already has similar utility commands (`init`, `clean`, `events`) and the `EventLogger` already handles proper JSON serialization via `serde_json`.

## Reference Documentation
**Required:**
- CLI structure: `crates/ralph-cli/src/main.rs` (Commands enum at ~line 195)
- Event logger: `crates/ralph-core/src/event_logger.rs` (EventRecord, EventLogger)

**Additional References:**
- Existing commands for patterns: `ralph init`, `ralph clean`, `ralph events`
- Event format: `docs/guide/agents.md` (lines 668-720)

**Note:** Study the existing command patterns in main.rs before implementation.

## Technical Requirements
1. Add `Emit(EmitArgs)` variant to `Commands` enum
2. Create `EmitArgs` struct with clap derive macros
3. Implement `emit_command()` handler function
4. Reuse `EventLogger` for file writing (handles directory creation, serialization)
5. Support both string and JSON object payloads
6. Auto-generate ISO 8601 timestamp if not provided
7. Default to `.agent/events.jsonl` path

## Dependencies
- `clap` - Already in use for CLI argument parsing
- `serde_json` - Already in workspace for JSON serialization
- `chrono` - Already in workspace for timestamp generation
- `EventLogger` from `ralph-core` - Already handles JSONL writing

## Implementation Approach

### 1. Define EmitArgs struct
```rust
#[derive(Parser, Debug)]
pub struct EmitArgs {
    /// Event topic (e.g., "build.done", "review.complete")
    pub topic: String,

    /// Event payload - string or JSON (optional)
    #[arg(default_value = "")]
    pub payload: String,

    /// Parse payload as JSON object instead of string
    #[arg(long, short)]
    pub json: bool,

    /// Custom ISO 8601 timestamp (defaults to now)
    #[arg(long)]
    pub ts: Option<String>,

    /// Path to events file (defaults to .agent/events.jsonl)
    #[arg(long, default_value = ".agent/events.jsonl")]
    pub file: PathBuf,
}
```

### 2. Add to Commands enum
```rust
enum Commands {
    // ... existing variants
    /// Emit an event to .agent/events.jsonl
    Emit(EmitArgs),
}
```

### 3. Implement handler
```rust
fn emit_command(args: EmitArgs) -> Result<()> {
    let ts = args.ts.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let payload = if args.json {
        // Validate and pass through as-is
        serde_json::from_str::<serde_json::Value>(&args.payload)
            .context("Invalid JSON payload")?;
        args.payload
    } else {
        args.payload
    };

    let mut logger = EventLogger::new(&args.file);
    // Write event using EventLogger (handles escaping, formatting)
    // ...

    println!("✓ Event emitted: {}", args.topic);
    Ok(())
}
```

## Acceptance Criteria

1. **Basic String Payload**
   - Given `ralph emit build.done "tests passed"`
   - When the command executes
   - Then `.agent/events.jsonl` contains valid single-line JSON with topic "build.done"

2. **Multi-line String Payload Escaped**
   - Given `ralph emit analysis.done "line1\nline2\nline3"`
   - When the command executes
   - Then the payload contains escaped `\n` characters, not literal newlines

3. **JSON Object Payload**
   - Given `ralph emit review.done --json '{"status":"approved","count":3}'`
   - When the command executes
   - Then `.agent/events.jsonl` contains the structured payload as a JSON object

4. **Auto-Generated Timestamp**
   - Given `ralph emit build.done "done"` without --ts flag
   - When the command executes
   - Then the event has a valid ISO 8601 timestamp close to current time

5. **Custom Timestamp**
   - Given `ralph emit build.done "done" --ts "2026-01-15T10:00:00Z"`
   - When the command executes
   - Then the event has exactly that timestamp

6. **Custom File Path**
   - Given `ralph emit build.done "done" --file /tmp/events.jsonl`
   - When the command executes
   - Then the event is written to `/tmp/events.jsonl`

7. **Directory Creation**
   - Given `.agent/` directory does not exist
   - When `ralph emit build.done "done"` executes
   - Then `.agent/` is created and event is written

8. **Invalid JSON Rejected**
   - Given `ralph emit test --json '{invalid json}'`
   - When the command executes
   - Then it exits with error "Invalid JSON payload"

9. **Help Text**
   - Given `ralph emit --help`
   - When executed
   - Then it shows usage with all options documented

10. **Integration with EventReader**
    - Given events written via `ralph emit`
    - When `EventReader::read_new_events()` is called
    - Then all emitted events are successfully parsed

## Metadata
- **Complexity**: Low
- **Labels**: CLI, Events, DX, Deterministic, Helper Tool
- **Required Skills**: Rust, clap, serde_json, CLI design
