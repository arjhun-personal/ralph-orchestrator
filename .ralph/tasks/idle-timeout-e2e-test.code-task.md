---
status: completed
created: 2026-01-14
started: 2026-01-14
completed: 2026-01-14
---
# Task: E2E Idle Timeout Validation with TUI Capture

## Description
Create a live end-to-end test that validates the Ralph orchestrator's idle timeout behavior in interactive mode. The test runs the orchestrator with a short timeout, captures the TUI state using `freeze`, and validates the visual output using Claude's Haiku model as an LLM-as-judge.

## Background
The Ralph orchestrator has idle timeout detection in interactive mode (`-i` flag) that terminates processes after a period of inactivity (no PTY output AND no user input). This is a critical feature for preventing hung sessions. Currently, there's no automated test that validates:
1. The timeout actually triggers after the configured period
2. The TUI correctly displays the termination reason
3. The visual output maintains proper hierarchy without artifacts

The Claude Agent SDK provides programmatic access to Claude models with the same tools and capabilities as Claude Code, making it ideal for LLM-as-judge validation.

## Reference Documentation
**Required:**
- CLAUDE.md - Contains TUI validation guidelines and freeze usage
- `crates/ralph-adapters/src/pty_executor.rs` (lines 905-1067) - Idle detection logic
- `crates/ralph-cli/src/main.rs` (lines 230-243, 1388-1392) - CLI flag handling
- `crates/ralph-core/src/config.rs` (lines 565-596) - Timeout configuration

**Additional References:**
- [Claude Agent SDK Quickstart](https://platform.claude.com/docs/en/agent-sdk/quickstart)
- [Claude Agent SDK Python Reference](https://platform.claude.com/docs/en/agent-sdk/python)

**Note:** Review the existing TUI validation skill at `.claude/skills/tui-validate/` for patterns.

## Technical Requirements
1. Create Python test script at `tools/e2e/test_idle_timeout.py`
2. Use `pytest` as the test framework with async support (`pytest-asyncio`)
3. Use Claude Agent SDK (`pip install claude-agent-sdk`) with local Claude Code auth
4. Use `tmux` for controlled terminal session management
5. Use `freeze` CLI tool for terminal capture to SVG/PNG
6. Use Haiku model (`claude-haiku-4-20250514`) for fast, cheap LLM-as-judge validation
7. Store captured evidence in `tui-validation/idle-timeout/` directory
8. Support configurable timeout duration (default: 5 seconds for fast testing)

## Dependencies
- Python 3.10+
- `claude-agent-sdk` - Claude Agent SDK for LLM-as-judge
- `pytest` and `pytest-asyncio` - Test framework
- `tmux` - Terminal multiplexer for session control
- `freeze` (charmbracelet) - Terminal screenshot tool (`brew install charmbracelet/tap/freeze`)
- Built Ralph binary (`cargo build --release`)

## Implementation Approach

### 1. Project Setup
```
tools/e2e/
├── __init__.py
├── conftest.py           # pytest fixtures (tmux session, freeze capture)
├── test_idle_timeout.py  # Main test file
├── helpers/
│   ├── __init__.py
│   ├── tmux.py          # Tmux session management utilities
│   ├── freeze.py        # Freeze capture utilities
│   └── llm_judge.py     # Claude Agent SDK validation helpers
└── requirements.txt      # Python dependencies
```

### 2. Tmux Session Management
- Create named tmux session with fixed dimensions (100x30)
- Send commands to session via `tmux send-keys`
- Capture pane content with `tmux capture-pane -p -e` (preserve ANSI)
- Clean up session after test

### 3. Test Flow
```
1. Build Ralph (if needed)
2. Create tmux session
3. Start Ralph: `ralph run --tui --idle-timeout 5 -c ralph.yml -p "Say hello"`
4. Wait for initial output (Claude responds)
5. Wait for idle timeout to trigger (~5-7 seconds after last output)
6. Capture TUI state with freeze
7. Validate with LLM-as-judge
8. Assert and cleanup
```

### 4. LLM-as-Judge Validation
Use Claude Agent SDK's `query()` function with Haiku to validate:
```python
from claude_agent_sdk import query

async def validate_tui(screenshot_path: str) -> tuple[bool, str]:
    prompt = f"""Analyze this TUI screenshot and validate:
    1. Shows idle timeout termination (look for "idle" or "timeout" in status)
    2. Header shows iteration count and elapsed time
    3. Footer shows activity indicator
    4. No visual artifacts or broken borders
    5. Proper visual hierarchy maintained

    Respond with JSON: {{"pass": true/false, "reason": "explanation"}}
    """

    async for msg in query(
        prompt=prompt,
        model="claude-haiku-4-20250514",
        # SDK uses local claude auth automatically
    ):
        # Parse response
        ...
```

### 5. Evidence Collection
- Save raw tmux capture (`.txt`)
- Save freeze SVG output (`.svg`)
- Save freeze PNG output (`.png`)
- Save LLM-as-judge response (`.json`)

## Acceptance Criteria

1. **Test Infrastructure Setup**
   - Given the `tools/e2e/` directory structure
   - When running `pip install -r tools/e2e/requirements.txt`
   - Then all dependencies install successfully including claude-agent-sdk

2. **Tmux Session Management**
   - Given no existing tmux session named "ralph-e2e-test"
   - When the test creates a session with fixed dimensions
   - Then commands can be sent and output captured reliably

3. **Idle Timeout Triggers Correctly**
   - Given Ralph running with `--idle-timeout 5`
   - When Claude completes its response and no further activity occurs
   - Then the process terminates within 7 seconds (5s timeout + 2s buffer)

4. **TUI Capture with Freeze**
   - Given a running Ralph TUI session in tmux
   - When `freeze` captures the terminal output
   - Then valid SVG and PNG files are created in `tui-validation/idle-timeout/`

5. **LLM-as-Judge Validation**
   - Given a captured TUI screenshot showing idle timeout
   - When validated using Claude Haiku via the Agent SDK
   - Then the response correctly identifies pass/fail with reasoning

6. **Evidence Preservation**
   - Given a completed test run (pass or fail)
   - When checking `tui-validation/idle-timeout/`
   - Then timestamped evidence files exist (txt, svg, png, json)

7. **Pytest Integration**
   - Given the test file at `tools/e2e/test_idle_timeout.py`
   - When running `pytest tools/e2e/ -v`
   - Then tests execute with clear pass/fail output and captured evidence

8. **Local Auth Works**
   - Given the user has authenticated Claude Code (run `claude` previously)
   - When the test uses Claude Agent SDK
   - Then no additional API key configuration is required

## Metadata
- **Complexity**: Medium
- **Labels**: E2E Testing, TUI Validation, LLM-as-Judge, Claude Agent SDK, Idle Timeout
- **Required Skills**: Python async programming, pytest, tmux, subprocess management, Claude Agent SDK
