---
status: completed
created: 2026-01-20
started: 2026-01-20
completed: 2026-01-20
---
# Task: Implement Run-Isolated Events File

## Description
Implement unique events file per Ralph run to prevent stale events from previous runs polluting new runs. Each run will use a timestamped events file (e.g., `.ralph/events-20260120-193202.jsonl`) with a marker file (`.ralph/current-events`) to coordinate between Ralph and the `ralph emit` command.

**Directory separation:**
- `.ralph/` → Orchestrator metadata (events, session markers, logs)
- `.agent/` → Agent-facing state (scratchpad, context for LLM)

## Background
Issue #82 reports that the opencode backend fails without useful error information. Investigation revealed that stale events from previous runs (with different configs) remain in `.agent/events.jsonl` and pollute new runs. Events with unrecognized topics (e.g., `archaeology.start`, `map.created` from a previous preset) cause "no subscriber" errors, leading to consecutive failure detection and loop termination.

The root cause is that all runs share a single events file, and the `EventReader` starts at position 0, reading ALL events including stale ones from previous runs.

**Current architecture (before this change):**
- Events file path hardcoded as `.agent/events.jsonl` in multiple places
- `EventReader` starts at position 0 on each run
- No cleanup of stale events between runs
- `ralph emit` command writes to the same shared file

**Proposed solution:**
- Move events to `.ralph/` directory (orchestrator metadata, separate from agent context)
- Generate unique timestamped events file per run (e.g., `.ralph/events-20260120-193202.jsonl`)
- Use marker file (`.ralph/current-events`) to coordinate path between Ralph and agents
- Preserve existing marker file for `ralph resume` scenarios

## Technical Requirements
1. Generate unique events filename at run startup using timestamp format `events-YYYYMMDD-HHMMSS.jsonl`
2. Write the events file path to `.ralph/current-events` marker file at run startup
3. Modify `EventLoop::new()` to read events path from marker file (with fallback to default)
4. Modify `emit_command()` to read events path from marker file (with fallback to CLI arg)
5. For `ralph resume`, reuse the existing marker file (do not generate new events file)
6. Ensure backward compatibility when marker file doesn't exist

## Dependencies
- `chrono` crate for timestamp generation (already a dependency)
- `std::fs` for file operations
- Existing `EventReader` and `EventLogger` infrastructure

## Implementation Approach

### 1. Modify run startup (`crates/ralph-cli/src/main.rs` in `run_loop_impl`)
```rust
// For fresh runs (not resume), generate unique events file
if !resume {
    let run_id = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let events_path = format!(".ralph/events-{}.jsonl", run_id);

    fs::create_dir_all(".ralph")?;
    fs::write(".ralph/current-events", &events_path)?;
    debug!("Created events file for this run: {}", events_path);
}
```

### 2. Modify EventLoop (`crates/ralph-core/src/event_loop.rs`)
```rust
// In EventLoop::new(), read current events path from marker
let events_path = fs::read_to_string(".ralph/current-events")
    .map(|s| s.trim().to_string())
    .unwrap_or_else(|_| ".ralph/events.jsonl".to_string());
let event_reader = EventReader::new(&events_path);
```

### 3. Modify emit command (`crates/ralph-cli/src/main.rs` in `emit_command`)
```rust
// Read current events path from marker, fall back to CLI arg
let events_file = fs::read_to_string(".ralph/current-events")
    .map(|s| PathBuf::from(s.trim()))
    .unwrap_or_else(|_| args.file.clone());
```

## Acceptance Criteria

1. **Fresh Run Creates Unique Events File**
   - Given a fresh `ralph run` command (not resume)
   - When the run starts
   - Then a new events file is created with timestamp in name (e.g., `.ralph/events-20260120-193202.jsonl`)
   - And the path is written to `.ralph/current-events`

2. **Events Written to Correct File**
   - Given an active Ralph run with unique events file
   - When an agent calls `ralph emit "topic" "payload"`
   - Then the event is written to the run-specific events file (not the default)

3. **Events Read from Correct File**
   - Given an active Ralph run with unique events file
   - When Ralph processes events from JSONL
   - Then it reads from the run-specific events file

4. **Resume Uses Existing Events File**
   - Given a previous run that was interrupted
   - When `ralph resume` is called
   - Then it uses the existing `.ralph/current-events` marker
   - And does NOT create a new events file

5. **Backward Compatibility Without Marker**
   - Given no `.ralph/current-events` marker file exists
   - When Ralph runs or `ralph emit` is called
   - Then it falls back to `.ralph/events.jsonl` (default behavior)

6. **Stale Events Isolation**
   - Given a previous run that wrote events with topics `archaeology.start`, `map.created`
   - When a new run starts with a different config
   - Then the new run does NOT see the stale events from the previous run

7. **Unit Tests Pass**
   - Given the implementation changes
   - When `cargo test` is run
   - Then all existing tests pass
   - And new tests cover the marker file functionality

## Test Cases

### Manual Testing
```bash
# Test 1: Fresh run creates unique events file
ralph run -p "test prompt" -v
# Verify: .ralph/current-events exists and points to timestamped file

# Test 2: Events written to correct file
ralph emit "test.event" "payload"
# Verify: Event appears in the timestamped file, not events.jsonl

# Test 3: Resume uses existing file
# (interrupt previous run, then)
ralph resume
# Verify: Uses same events file from previous run

# Test 4: Backward compatibility
rm .ralph/current-events
ralph emit "test.event" "payload"
# Verify: Falls back to .ralph/events.jsonl
```

### Smoke Test
Run the existing smoke tests to ensure no regressions:
```bash
cargo test -p ralph-core smoke_runner
```

## Files to Modify

| File | Changes |
|------|---------|
| `crates/ralph-cli/src/main.rs` | Add marker file creation in `run_loop_impl()`, modify `emit_command()` to read marker, update default path in `EmitArgs` |
| `crates/ralph-core/src/event_loop.rs` | Modify `EventLoop::new()` to read events path from marker |
| `crates/ralph-core/src/event_logger.rs` | Update `DEFAULT_PATH` constant from `.agent/events.jsonl` to `.ralph/events.jsonl` |
| `crates/ralph-core/src/event_reader.rs` | Update module doc comments to reference `.ralph/` |

## Metadata
- **Complexity**: Medium
- **Labels**: Bug Fix, Events, Architecture, Issue-82
- **Required Skills**: Rust, File I/O, Event System
- **Related Issue**: https://github.com/mikeyobrien/ralph-orchestrator/issues/82
