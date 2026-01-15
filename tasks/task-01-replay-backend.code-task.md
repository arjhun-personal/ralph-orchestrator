---
status: completed
created: 2026-01-14
started: 2026-01-14
completed: 2026-01-14
---
# Task: Create Replay Backend for Claude CLI Output

## Description
Create a `ReplayBackend` that reads recorded JSONL session files (from `SessionRecorder`) and replays them as mock CLI responses. This enables deterministic smoketesting without live Claude API calls by using previously recorded sessions as test fixtures.

## Background
The codebase has robust session recording (`SessionRecorder`) and playback (`SessionPlayer`) infrastructure that captures Claude CLI output as JSONL. However, there's currently no way to use these recordings as test fixtures to drive the event loop. A replay backend would bridge this gap, allowing recorded sessions to be "played back" through the normal event loop pipeline for deterministic testing.

The existing `MockBackend` in `crates/ralph-core/src/testing/mock_backend.rs` provides scripted responses but doesn't integrate with the JSONL recording format or simulate realistic streaming behavior.

## Technical Requirements
1. Create a `ReplayBackend` struct that implements the same interface pattern as the CLI backend
2. Load JSONL fixture files using the existing `SessionPlayer` infrastructure
3. Extract and serve terminal write events (`ux.terminal.write`) in sequence when polled
4. Support configurable replay timing (instant for fast tests, or realistic delays)
5. Provide clear error messages when fixture data is exhausted or malformed
6. Integrate with the existing `CliCapture` or provide an equivalent streaming interface

## Dependencies
- `crates/ralph-core/src/session_player.rs` - for JSONL parsing
- `crates/ralph-core/src/session_recorder.rs` - for `Record` type
- `crates/ralph-adapters/src/cli_backend.rs` - for backend interface patterns
- `ralph_proto::UxEvent` and `ralph_proto::TerminalWrite` types

## Implementation Approach
1. Study the `CliBackend` interface in `crates/ralph-adapters/src/cli_backend.rs` to understand expected behavior
2. Create `ReplayBackend` in `crates/ralph-core/src/testing/replay_backend.rs`
3. Use `SessionPlayer::from_reader()` to load fixture data
4. Implement a method that yields terminal write bytes in sequence (simulating streaming output)
5. Add configuration for instant vs timed replay modes
6. Export from `crates/ralph-core/src/testing/mod.rs`

## Acceptance Criteria

1. **Load JSONL Fixture**
   - Given a valid JSONL file recorded by `SessionRecorder`
   - When `ReplayBackend::from_file(path)` is called
   - Then the backend loads and parses all records without error

2. **Serve Terminal Output**
   - Given a loaded replay backend with terminal write events
   - When output is requested via the streaming interface
   - Then terminal write bytes are returned in recorded order

3. **Handle Empty/Missing Fixtures**
   - Given a non-existent file path or empty fixture
   - When attempting to create a replay backend
   - Then a descriptive error is returned (not a panic)

4. **Instant Replay Mode**
   - Given a replay backend configured for instant mode
   - When replaying a multi-second recording
   - Then all output is served immediately without timing delays

5. **Exhaust Detection**
   - Given a replay backend that has served all recorded output
   - When more output is requested
   - Then an appropriate EOF/completion signal is returned

6. **Unit Test Coverage**
   - Given the replay backend implementation
   - When running the test suite
   - Then all core scenarios have corresponding unit tests

## Metadata
- **Complexity**: Medium
- **Labels**: Testing, Backend, Replay, JSONL, Fixtures
- **Required Skills**: Rust async/streaming patterns, JSONL parsing, test infrastructure design
