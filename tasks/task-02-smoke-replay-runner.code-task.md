---
status: completed
created: 2026-01-14
started: 2026-01-14
completed: 2026-01-14
---
# Task: Create Smoke Test Replay Runner

## Description
Create a test runner/harness that loads JSONL fixture files and runs the Ralph event loop with the `ReplayBackend`, verifying expected behaviors. This enables CI-friendly smoke tests that validate event parsing, signal handling, and hat behaviors without requiring live Claude API calls.

## Background
With the `ReplayBackend` (task-01) in place, we need a way to orchestrate smoke tests using recorded sessions. This runner should:
- Load fixture files from a designated directory (e.g., `fixtures/` or `tests/fixtures/`)
- Configure and run the event loop with replay mode
- Assert on expected outcomes (completion signals, event counts, parsed events)

This completes the replay-based testing story: record once with real Claude → replay infinitely in CI.

## Technical Requirements
1. Create a test harness that wires `ReplayBackend` into the event loop
2. Support loading fixtures by name from a configurable directory
3. Provide assertion helpers for common smoke test scenarios
4. Return structured results including: iterations run, events parsed, termination reason
5. Support both programmatic use (in `#[test]` functions) and potential CLI invocation
6. Handle fixture discovery (list available fixtures, validate format)

## Dependencies
- `ReplayBackend` from task-01
- `crates/ralph-core/src/event_loop.rs` - event loop implementation
- `crates/ralph-core/src/event_parser.rs` - for verifying parsed events
- `crates/ralph-core/src/config.rs` - for test configuration

## Implementation Approach
1. Create `crates/ralph-core/src/testing/smoke_runner.rs`
2. Define a `SmokeTestConfig` struct with fixture path, expected outcomes, timeout
3. Implement `run_smoke_test(config) -> SmokeTestResult` that:
   - Creates a `ReplayBackend` from the fixture
   - Configures a minimal event loop (no real CLI, no file I/O side effects)
   - Runs to completion or timeout
   - Collects metrics and parsed events
4. Add assertion helper methods on `SmokeTestResult`
5. Create example fixture(s) in `tests/fixtures/` demonstrating the format
6. Add integration test demonstrating full replay flow

## Acceptance Criteria

1. **Run Fixture Through Event Loop**
   - Given a valid JSONL fixture with a complete session
   - When `run_smoke_test()` is called with the fixture path
   - Then the event loop processes all events and returns a result

2. **Capture Termination Reason**
   - Given a fixture that ends with a completion promise
   - When the smoke test completes
   - Then the result includes the correct termination reason

3. **Event Counting**
   - Given a fixture with known event counts
   - When the smoke test completes
   - Then `result.event_count()` matches expected values

4. **Timeout Handling**
   - Given a fixture and a short timeout configuration
   - When replay would exceed the timeout
   - Then the test terminates gracefully with a timeout result

5. **Fixture Not Found**
   - Given a non-existent fixture path
   - When attempting to run the smoke test
   - Then a clear error is returned (not a panic)

6. **Example Fixture Included**
   - Given the implementation is complete
   - When checking the repository
   - Then at least one example fixture exists in `tests/fixtures/` with documentation

7. **Integration Test**
   - Given the smoke runner implementation
   - When running `cargo test`
   - Then at least one integration test validates the full replay flow

## Metadata
- **Complexity**: Medium
- **Labels**: Testing, Smoke Test, Integration, Event Loop, CI
- **Required Skills**: Rust testing patterns, event loop integration, test fixture design
