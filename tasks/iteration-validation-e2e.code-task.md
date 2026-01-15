---
status: completed
created: 2026-01-14
started: 2026-01-14
completed: 2026-01-14
---
# Task: E2E Multi-Iteration Validation with TUI Capture

## Description
Create a live end-to-end test suite that validates Ralph's iteration lifecycle behaviors as defined in the event-loop spec. The tests capture TUI state across multiple iterations and use LLM-as-judge to verify iteration counter increments, fresh context per iteration, max iterations termination, and dual confirmation completion.

## Background
The Ralph orchestrator's iteration system is the core loop mechanism. Per **Tenet #1 (Fresh Context Is Reliability)**, each iteration clears context and re-reads the scratchpad. The spec defines:

- **Iteration Counter**: 1-indexed, increments exactly once per `process_output()` call
- **Fresh Context**: Scratchpad injected fresh into prompt each iteration (no agent state carryover)
- **Max Iterations**: Terminates with exit code 2 when `iteration >= max_iterations`
- **Dual Confirmation**: Completion requires 2 consecutive `LOOP_COMPLETE` outputs from Ralph with scratchpad verification
- **Consecutive Failures**: Exit code 1 after 5 consecutive failures

Currently, unit tests exist in `crates/ralph-core/tests/event_loop_ralph.rs` but no live E2E test validates the TUI correctly displays iteration state during multi-iteration runs.

## Reference Documentation
**Required:**
- `specs/event-loop/design/detailed-design.md` (lines 149-185) - Iteration Lifecycle
- `crates/ralph-core/src/event_loop.rs` - LoopState struct, check_termination()
- `crates/ralph-core/src/config.rs` (lines 446-475) - max_iterations, max_runtime_seconds

**Additional References:**
- `tools/e2e/` - Existing E2E test infrastructure (tmux, freeze, LLM judge)
- `crates/ralph-core/tests/event_loop_ralph.rs` - Unit test patterns

## Technical Requirements
1. Extend existing `tools/e2e/` infrastructure with new test file `test_iteration_lifecycle.py`
2. Create helper to capture TUI state at iteration boundaries (poll for `[iter N]` changes)
3. Create configurable test scenarios:
   - **Counter Test**: Run 3-5 iterations, capture each, validate counter increments
   - **Max Iterations Test**: Set `max_iterations=3`, verify exits with code 2
   - **Completion Test**: Task that completes in 2 iterations, verify dual confirmation
   - **Fresh Context Test**: Verify scratchpad content changes between iterations
4. Use Haiku model for LLM-as-judge validation of each captured state
5. Store timestamped evidence in `tui-validation/iteration-lifecycle/`
6. Support `--iteration-delay` parameter to control capture timing (default: 2s)

## Dependencies
- Existing `tools/e2e/` infrastructure (TmuxSession, FreezeCapture, LLMJudge)
- Built Ralph binary with TUI support
- Ralph config with configurable `max_iterations`
- Simple test prompt that produces predictable multi-iteration behavior

## Implementation Approach

### 1. Test Scenarios

**Scenario A: Iteration Counter Validation**
```python
async def test_iteration_counter_increments():
    """Validate TUI shows [iter 1], [iter 2], [iter 3] in sequence."""
    # Run Ralph with prompt requiring multiple iterations
    # Capture TUI at each iteration boundary
    # LLM-judge validates each capture shows correct [iter N]
```

**Scenario B: Max Iterations Termination**
```python
async def test_max_iterations_exit_code():
    """Validate loop terminates at max_iterations with exit code 2."""
    # Run with max_iterations=3 and long-running task
    # Wait for termination
    # Assert exit code == 2
    # Capture final TUI state
    # LLM-judge validates shows 3 iterations completed
```

**Scenario C: Dual Confirmation Completion**
```python
async def test_completion_dual_confirmation():
    """Validate completion requires 2 consecutive LOOP_COMPLETE."""
    # Run simple task that completes quickly
    # Capture TUI during completion phase
    # LLM-judge validates shows completion state
    # Assert exit code == 0
```

**Scenario D: Fresh Context Per Iteration**
```python
async def test_fresh_context_scratchpad_reread():
    """Validate scratchpad is re-read each iteration (not cached)."""
    # Run task, modify scratchpad externally mid-iteration
    # Capture subsequent iteration
    # LLM-judge validates new scratchpad content reflected
```

### 2. Iteration Capture Helper

```python
class IterationCapture:
    """Helper to capture TUI state at iteration boundaries."""

    async def wait_for_iteration(self, n: int, timeout: float = 30.0) -> str:
        """Wait until TUI shows [iter N] and capture state."""
        # Poll tmux capture for [iter N] pattern
        # Return captured content when found

    async def capture_sequence(self, max_iter: int) -> list[CaptureResult]:
        """Capture TUI state for iterations 1 through max_iter."""
        # Returns list of captures, one per iteration
```

### 3. LLM-as-Judge Criteria

```python
ITERATION_COUNTER_CRITERIA = """
Analyze this TUI capture and validate:

1. **Iteration Display**: Header shows iteration in format [iter N]
   - Extract the iteration number N
   - Verify it matches expected value: {expected_iteration}

2. **Elapsed Time**: Header shows elapsed time in MM:SS format
   - Time should be > 00:00
   - Time should increase between iterations

3. **Mode Indicator**: Shows current mode (auto/interactive/etc)
   - Should be visible in header area

4. **No Artifacts**: TUI renders without visual corruption
   - Borders are complete
   - Text is readable

Respond with JSON:
{
  "pass": true/false,
  "iteration_found": <number or null>,
  "elapsed_time": "<MM:SS or null>",
  "checks": {
    "iteration_correct": {"pass": true/false, "reason": "..."},
    "elapsed_visible": {"pass": true/false, "reason": "..."},
    "mode_visible": {"pass": true/false, "reason": "..."},
    "no_artifacts": {"pass": true/false, "reason": "..."}
  },
  "overall_reason": "..."
}
"""

MAX_ITERATIONS_CRITERIA = """
Analyze this TUI capture from a terminated session:

1. **Final Iteration**: Shows the max iteration count reached
   - Should show [iter {max_iterations}] or indication of limit reached

2. **Termination Reason**: Shows why loop terminated
   - Look for: "max iterations", "limit reached", or similar

3. **Session Complete**: No active processing indicators
   - Activity indicator should show stopped (■) or similar

Respond with JSON: {...}
"""
```

### 4. Test Config Generation

```python
def create_iteration_test_config(
    max_iterations: int = 100,
    max_runtime_seconds: int = 300,
) -> Path:
    """Create a Ralph config file for iteration testing."""
    config = f"""
cli:
  backend: claude
  default_mode: autonomous

orchestrator:
  max_iterations: {max_iterations}
  max_runtime_seconds: {max_runtime_seconds}
  max_consecutive_failures: 5
"""
    # Write to temp file and return path
```

### 5. Evidence Structure

```
tui-validation/iteration-lifecycle/
├── run_20260115_120000/
│   ├── scenario_counter/
│   │   ├── iter_1_capture.txt
│   │   ├── iter_1_capture.svg
│   │   ├── iter_1_judge.json
│   │   ├── iter_2_capture.txt
│   │   ├── iter_2_capture.svg
│   │   ├── iter_2_judge.json
│   │   └── ...
│   ├── scenario_max_iterations/
│   │   ├── final_capture.txt
│   │   ├── final_capture.svg
│   │   ├── judge_result.json
│   │   └── exit_code.txt
│   └── scenario_completion/
│       └── ...
```

## Acceptance Criteria

1. **Iteration Counter Test**
   - Given Ralph running a multi-iteration task
   - When TUI is captured at iterations 1, 2, 3
   - Then each capture shows correct `[iter N]` in header
   - And LLM-judge validates counter increments by exactly 1

2. **Max Iterations Termination**
   - Given Ralph config with `max_iterations: 3`
   - When running a task that would require >3 iterations
   - Then loop terminates after iteration 3
   - And exit code is 2 (per spec)
   - And TUI capture shows termination state

3. **Dual Confirmation Completion**
   - Given Ralph running a simple completable task
   - When task completes with LOOP_COMPLETE
   - Then loop requires 2 consecutive confirmations (per spec)
   - And exit code is 0
   - And TUI shows completion state

4. **Fresh Context Validation**
   - Given Ralph in iteration N
   - When scratchpad is modified externally before iteration N+1
   - Then iteration N+1 prompt includes updated scratchpad content
   - And LLM-judge can detect the change in TUI output

5. **Evidence Preservation**
   - Given any test scenario execution
   - When test completes (pass or fail)
   - Then `tui-validation/iteration-lifecycle/` contains:
     - Timestamped capture files (txt, svg)
     - LLM-judge results (json)
     - Exit code record

6. **Pytest Integration**
   - Given the test file at `tools/e2e/test_iteration_lifecycle.py`
   - When running `pytest tools/e2e/test_iteration_lifecycle.py -v`
   - Then all scenarios execute with clear pass/fail output

7. **Exit Code Verification**
   - Given each termination reason
   - When loop terminates
   - Then exit code matches spec:
     - 0 = Completed (LOOP_COMPLETE)
     - 1 = Stopped/ConsecutiveFailures
     - 2 = MaxIterations/MaxRuntime/MaxCost
     - 130 = Interrupted (SIGINT)

## Metadata
- **Complexity**: Medium-High
- **Labels**: E2E Testing, TUI Validation, Iteration Lifecycle, LLM-as-Judge, Event Loop
- **Required Skills**: Python async, pytest, tmux, subprocess management, Ralph internals
