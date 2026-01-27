---
status: completed
created: 2026-01-15
started: 2026-01-15
completed: 2026-01-15
---
# Task: Add Event Validation Backpressure for Malformed JSONL

## Description
Add deterministic validation backpressure to the EventReader that emits `event.malformed` system events when JSONL parsing fails. This provides feedback to agents about format errors and terminates the loop after consecutive validation failures, following the "Backpressure Over Prescription" tenet.

## Background
Currently, `EventReader::read_new_events()` silently skips malformed JSON lines with only a warning log. Agents receive no feedback that their events were rejected, leading to silent failures.

The codebase already implements backpressure for `build.done` events (synthesizing `build.blocked` on validation failure). This task extends that pattern to JSONL parsing, making malformed events trigger a deterministic response rather than silent drops.

## Reference Documentation
**Required:**
- EventReader: `crates/ralph-core/src/event_reader.rs` (lines 71-104, read_new_events)
- EventLoop: `crates/ralph-core/src/event_loop.rs` (LoopState, TerminationReason, process_events_from_jsonl)

**Additional References:**
- Existing backpressure pattern: `event_loop.rs` lines 593-630 (build.done validation)
- LoopState fields: `event_loop.rs` lines 76-101

**Note:** Study the existing `build.done` → `build.blocked` pattern before implementing.

## Technical Requirements
1. Extend `EventReader` to return both valid events and parse errors
2. Add `MalformedLine` struct to capture error details
3. Add `consecutive_malformed_events` counter to `LoopState`
4. Add `TerminationReason::ValidationFailure` variant
5. Emit `event.malformed` system event for each parse failure
6. Terminate loop after 3 consecutive malformed events
7. Reset counter when valid events are parsed

## Dependencies
- No new dependencies - uses existing EventBus, LoopState patterns
- `serde` and `serde_json` already in use

## Implementation Approach

### 1. Add ParseResult and MalformedLine types (event_reader.rs)
```rust
/// Result of parsing events from JSONL file.
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Successfully parsed events.
    pub events: Vec<Event>,
    /// Lines that failed to parse.
    pub malformed: Vec<MalformedLine>,
}

/// Information about a malformed JSONL line.
#[derive(Debug, Clone, Serialize)]
pub struct MalformedLine {
    /// Line number in the file (1-indexed).
    pub line_number: u32,
    /// The raw content that failed to parse.
    pub content: String,
    /// The parse error message.
    pub error: String,
}
```

### 2. Update read_new_events() signature and implementation
```rust
// Change from:
pub fn read_new_events(&mut self) -> std::io::Result<Vec<Event>>
// To:
pub fn read_new_events(&mut self) -> std::io::Result<ParseResult>

// Collect errors instead of just warning:
Err(e) => {
    malformed.push(MalformedLine {
        line_number,
        content: line.clone(),
        error: e.to_string(),
    });
    warn!(...);
}
```

### 3. Add state tracking to LoopState (event_loop.rs)
```rust
pub struct LoopState {
    // ... existing fields ...
    /// Consecutive malformed event lines encountered.
    pub consecutive_malformed_events: u32,
}
```

### 4. Add TerminationReason variant
```rust
pub enum TerminationReason {
    // ... existing variants ...
    /// Too many consecutive malformed JSONL lines.
    ValidationFailure,
}

impl TerminationReason {
    pub fn exit_code(&self) -> i32 {
        match self {
            // ...
            Self::ValidationFailure => 1, // Failure category
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            // ...
            Self::ValidationFailure => "validation_failure",
        }
    }
}
```

### 5. Update process_events_from_jsonl() to emit system events
```rust
pub fn process_events_from_jsonl(&mut self) -> std::io::Result<bool> {
    let result = self.event_reader.read_new_events()?;

    // Handle malformed lines
    for malformed in &result.malformed {
        let payload = format!(
            "Line {}: {}\nContent: {}",
            malformed.line_number,
            malformed.error,
            truncate(&malformed.content, 100)
        );

        let event = Event::new("event.malformed", &payload);
        self.bus.publish(event);

        self.state.consecutive_malformed_events += 1;
    }

    // Reset on valid events
    if !result.events.is_empty() {
        self.state.consecutive_malformed_events = 0;
    }

    // Route valid events (existing logic)
    // ...
}
```

### 6. Add termination check
```rust
pub fn check_termination(&self) -> Option<TerminationReason> {
    // ... existing checks ...

    // Check for validation failures
    if self.state.consecutive_malformed_events >= 3 {
        return Some(TerminationReason::ValidationFailure);
    }

    None
}
```

## Acceptance Criteria

1. **ParseResult Return Type**
   - Given EventReader::read_new_events() is called
   - When there are both valid and malformed lines
   - Then it returns ParseResult with both events and malformed vectors populated

2. **MalformedLine Capture**
   - Given a malformed JSON line in events.jsonl
   - When read_new_events() encounters it
   - Then MalformedLine contains line_number, content, and error message

3. **System Event Emission**
   - Given a malformed line is detected
   - When process_events_from_jsonl() runs
   - Then an "event.malformed" event is published to the EventBus

4. **Consecutive Counter Increment**
   - Given 2 consecutive malformed lines
   - When process_events_from_jsonl() runs twice
   - Then state.consecutive_malformed_events equals 2

5. **Counter Reset on Valid Events**
   - Given consecutive_malformed_events is 2
   - When a valid event is successfully parsed
   - Then consecutive_malformed_events resets to 0

6. **Termination After 3 Failures**
   - Given 3 consecutive calls with malformed events only
   - When check_termination() is called
   - Then it returns Some(TerminationReason::ValidationFailure)

7. **Exit Code 1 for ValidationFailure**
   - Given TerminationReason::ValidationFailure
   - When exit_code() is called
   - Then it returns 1

8. **Reason String**
   - Given TerminationReason::ValidationFailure
   - When as_str() is called
   - Then it returns "validation_failure"

9. **Observer Notification**
   - Given event.malformed is published
   - When SessionRecorder is observing
   - Then the malformed event is recorded in the session

10. **Existing Tests Pass**
    - Given all changes are implemented
    - When cargo test runs
    - Then all existing EventReader and EventLoop tests pass

11. **Mixed Valid/Invalid Handling**
    - Given events.jsonl has: valid, invalid, valid lines
    - When read_new_events() runs
    - Then ParseResult.events has 2 items and ParseResult.malformed has 1 item

## Metadata
- **Complexity**: Medium
- **Labels**: Events, Backpressure, Validation, Deterministic, Error Handling
- **Required Skills**: Rust, event-driven architecture, error handling patterns
