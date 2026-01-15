---
status: completed
created: 2026-01-14
started: 2026-01-14
completed: 2026-01-14
---
# Task: Expand Smoke Test Infrastructure for Kiro CLI

## Description
Extend the replay-based smoke testing infrastructure to support Kiro CLI (`kiro-cli`) output recordings. This enables deterministic CI testing of Ralph's Kiro adapter without requiring AWS authentication or live API calls.

## Background
The smoke test infrastructure (ReplayBackend + SmokeRunner) currently works with Claude CLI JSONL recordings. Kiro CLI is AWS's coding assistant (formerly Amazon Q Developer CLI) and has different output characteristics:

- **Command**: `kiro-cli chat --no-interactive --trust-all-tools "prompt"`
- **Output format**: May differ from Claude's NDJSON streaming
- **Tool events**: Built-in tools include `read`, `write`, `shell`, `aws`, `report`
- **Behavior flags**: `--no-interactive`, `--trust-all-tools`

The existing infrastructure should be extended to:
1. Parse Kiro CLI output format (may require adapter)
2. Provide example Kiro session fixtures
3. Validate Kiro-specific behaviors (tool trust, autonomous mode)

## Technical Requirements
1. Analyze Kiro CLI output format and determine if ReplayBackend needs adaptation
2. Create Kiro-specific test fixtures in `tests/fixtures/kiro/`
3. Add smoke tests validating Kiro adapter behaviors per `specs/adapters/kiro.spec.md`
4. Support both autonomous mode (`--no-interactive`) and interactive mode recordings
5. Validate event parsing works correctly for Kiro's tool invocation output

## Dependencies
- `crates/ralph-core/src/testing/replay_backend.rs` - existing replay infrastructure
- `crates/ralph-core/src/testing/smoke_runner.rs` - existing smoke runner
- `specs/adapters/kiro.spec.md` - Kiro adapter specification
- `ralph.kiro.yml` - Kiro configuration example

## Implementation Approach
1. **Research Phase**: Record a real Kiro CLI session and analyze output format
   - Run `kiro-cli chat --no-interactive --trust-all-tools "echo hello"`
   - Capture output and compare to Claude format
2. **Adapter Phase** (if needed): Extend ReplayBackend to handle Kiro format differences
3. **Fixture Phase**: Create example Kiro session fixtures:
   - `basic_kiro_session.jsonl` - Simple prompt/response
   - `kiro_tool_use.jsonl` - Session with tool invocations
   - `kiro_completion.jsonl` - Session ending with LOOP_COMPLETE
4. **Test Phase**: Add integration tests for Kiro-specific behaviors:
   - Autonomous mode flag handling
   - Tool trust validation
   - Event parsing for Kiro's output structure
5. **Documentation**: Update fixture README with Kiro recording instructions

## Acceptance Criteria

1. **Kiro Output Format Supported**
   - Given a recorded Kiro CLI session in JSONL format
   - When loaded via ReplayBackend
   - Then terminal output is correctly extracted and served

2. **Kiro Fixtures Exist**
   - Given the implementation is complete
   - When checking `tests/fixtures/kiro/`
   - Then at least 2 example fixtures exist with documentation

3. **Autonomous Mode Validated**
   - Given a Kiro fixture from autonomous mode (--no-interactive)
   - When the smoke test runs
   - Then the result correctly identifies termination behavior

4. **Tool Invocation Events Parsed**
   - Given a Kiro fixture with tool use (read, write, shell)
   - When processed through the smoke runner
   - Then tool invocation events are counted correctly

5. **Cross-Backend Compatibility**
   - Given both Claude and Kiro fixtures
   - When running smoke tests for each
   - Then both pass using the same SmokeRunner infrastructure

6. **Recording Instructions Documented**
   - Given a developer wants to create new Kiro fixtures
   - When reading `tests/fixtures/kiro/README.md`
   - Then clear instructions explain how to record and format fixtures

7. **Integration Test Coverage**
   - Given the Kiro smoke test implementation
   - When running `cargo test`
   - Then at least one integration test validates Kiro replay flow

## Notes

### Recording Kiro Sessions
To create fixtures, you may need to:
```bash
# Enable session recording in Ralph
cargo run --bin ralph -- run -c ralph.kiro.yml --record-session session.jsonl -p "your prompt"

# Or capture raw output
kiro-cli chat --no-interactive --trust-all-tools "prompt" 2>&1 | tee kiro_output.txt
```

### Format Differences to Investigate
- Does Kiro use NDJSON streaming like Claude?
- Are terminal write events encoded the same way?
- Does Kiro emit different event types for tool use?

## Metadata
- **Complexity**: Medium
- **Labels**: Testing, Smoke Test, Kiro, CLI Adapter, Fixtures
- **Required Skills**: Rust, CLI output parsing, test fixture design, AWS Kiro CLI familiarity
