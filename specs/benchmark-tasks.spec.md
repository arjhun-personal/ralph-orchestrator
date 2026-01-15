---
status: implemented
gap_analysis: 2026-01-14
related:
  - benchmark-harness.spec.md
---

# Benchmark Task Suite Specification

## Overview

Define a minimal set of benchmark tasks with clear completion criteria for evaluating Ralph loop effectiveness. Tasks are graduated by complexity and have verifiable outcomes.

## Goals

1. **Graduated complexity**: Tasks range from trivial (1-3 iterations) to complex (10+ iterations)
2. **Verifiable outcomes**: Each task has a bash verification command
3. **Reproducible**: Tasks run in isolated directories with defined setup
4. **Extensible**: Simple JSON format for adding custom tasks

## Task Definition Schema

```json
{
  "name": "task-identifier",
  "description": "Human-readable description",
  "complexity": "simple|medium|complex",
  "prompt_file": "path/to/prompt.md",
  "completion_promise": "TASK_COMPLETE",
  "max_iterations": 10,
  "expected_iterations": 3,
  "timeout_seconds": 300,
  "setup": {
    "script": "setup.sh",
    "files": ["template.py"]
  },
  "verification": {
    "command": "pytest tests/ -q",
    "success_exit_code": 0
  },
  "tags": ["python", "testing", "tdd"]
}
```

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Unique task identifier (alphanumeric + hyphens) |
| `prompt_file` | string | Path to prompt markdown file |
| `completion_promise` | string | String agent outputs when done |
| `verification.command` | string | Bash command to verify success |

### Optional Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_iterations` | u32 | 100 | Safety limit |
| `expected_iterations` | u32 | null | Baseline for comparison |
| `timeout_seconds` | u64 | 300 | Per-task timeout |
| `setup.script` | string | null | Setup script to run before task |
| `setup.files` | string[] | [] | Files to copy to workspace |
| `complexity` | string | "medium" | simple, medium, or complex |
| `tags` | string[] | [] | Filtering/categorization |

## Starter Task Suite

### Simple Tasks (1-3 iterations expected)

#### hello-world
```json
{
  "name": "hello-world",
  "description": "Create a Python script that prints Hello World",
  "complexity": "simple",
  "prompt_file": "tasks/hello-world/PROMPT.md",
  "completion_promise": "TASK_COMPLETE",
  "max_iterations": 5,
  "expected_iterations": 1,
  "verification": {
    "command": "python hello.py | grep -q 'Hello, World!'"
  }
}
```

#### rename-function
```json
{
  "name": "rename-function",
  "description": "Rename a function across multiple files",
  "complexity": "simple",
  "prompt_file": "tasks/rename-function/PROMPT.md",
  "completion_promise": "REFACTOR_COMPLETE",
  "max_iterations": 5,
  "expected_iterations": 2,
  "setup": {
    "files": ["src/utils.py", "src/main.py", "tests/test_utils.py"]
  },
  "verification": {
    "command": "grep -r 'get_current_directory' src/ tests/ && ! grep -r 'getCwd' src/ tests/"
  }
}
```

### Medium Tasks (4-10 iterations expected)

#### fizzbuzz-tdd
```json
{
  "name": "fizzbuzz-tdd",
  "description": "Implement FizzBuzz with test-driven development",
  "complexity": "medium",
  "prompt_file": "tasks/fizzbuzz-tdd/PROMPT.md",
  "completion_promise": "TESTS_PASSING",
  "max_iterations": 15,
  "expected_iterations": 5,
  "setup": {
    "files": ["test_fizzbuzz.py"]
  },
  "verification": {
    "command": "pytest test_fizzbuzz.py -v"
  },
  "tags": ["python", "tdd"]
}
```

#### add-cli-flag
```json
{
  "name": "add-cli-flag",
  "description": "Add a new command-line flag to an existing CLI",
  "complexity": "medium",
  "prompt_file": "tasks/add-cli-flag/PROMPT.md",
  "completion_promise": "FLAG_IMPLEMENTED",
  "max_iterations": 15,
  "expected_iterations": 6,
  "setup": {
    "files": ["cli.py", "test_cli.py"]
  },
  "verification": {
    "command": "python cli.py --dry-run && pytest test_cli.py -v"
  },
  "tags": ["python", "cli"]
}
```

### Complex Tasks (10+ iterations expected)

#### fix-failing-tests
```json
{
  "name": "fix-failing-tests",
  "description": "Debug and fix multiple failing unit tests",
  "complexity": "complex",
  "prompt_file": "tasks/fix-failing-tests/PROMPT.md",
  "completion_promise": "ALL_TESTS_GREEN",
  "max_iterations": 30,
  "expected_iterations": 12,
  "setup": {
    "files": ["src/calculator.py", "tests/test_calculator.py"]
  },
  "verification": {
    "command": "pytest tests/ -v --tb=short"
  },
  "tags": ["python", "debugging", "testing"]
}
```

#### implement-feature
```json
{
  "name": "implement-feature",
  "description": "Implement a caching layer for an API client",
  "complexity": "complex",
  "prompt_file": "tasks/implement-feature/PROMPT.md",
  "completion_promise": "FEATURE_COMPLETE",
  "max_iterations": 50,
  "expected_iterations": 20,
  "setup": {
    "files": ["api_client.py", "test_api_client.py"]
  },
  "verification": {
    "command": "pytest test_api_client.py -v -k 'cache'"
  },
  "tags": ["python", "feature", "api"]
}
```

## Task Workspace Isolation

Each task runs in an isolated workspace:

```
/tmp/ralph-bench-{task-name}-{timestamp}/
├── PROMPT.md          # Copied from prompt_file
├── .agent/
│   └── scratchpad.md  # Fresh scratchpad
└── {setup.files}      # Copied from task definition
```

### Isolation Guarantees

1. Fresh directory per task run
2. No carryover from previous tasks
3. Git initialized (for agent commits)
4. Cleanup after verification (configurable)

## Metrics Collection

For each task run, collect:

| Metric | Type | Source |
|--------|------|--------|
| `task_name` | string | Task definition |
| `iterations` | u32 | `LoopState.iteration` |
| `duration_secs` | f64 | `LoopState.elapsed()` |
| `termination_reason` | string | `TerminationReason` enum |
| `verification_passed` | bool | Verification command exit code |
| `expected_iterations` | u32 | Task definition |
| `iteration_delta` | i32 | `iterations - expected_iterations` |

### Results Output

```json
{
  "run_id": "bench-20240101-120000",
  "timestamp": "2024-01-01T12:00:00Z",
  "tasks": [
    {
      "name": "hello-world",
      "iterations": 1,
      "expected_iterations": 1,
      "iteration_delta": 0,
      "duration_secs": 12.5,
      "termination_reason": "CompletionPromise",
      "verification_passed": true
    },
    {
      "name": "fizzbuzz-tdd",
      "iterations": 7,
      "expected_iterations": 5,
      "iteration_delta": 2,
      "duration_secs": 95.3,
      "termination_reason": "CompletionPromise",
      "verification_passed": true
    }
  ],
  "summary": {
    "total_tasks": 2,
    "passed": 2,
    "failed": 0,
    "total_iterations": 8,
    "total_duration_secs": 107.8
  }
}
```

## Acceptance Criteria

### Task Loading

- **Given** a valid tasks.json file
- **When** loaded by benchmark runner
- **Then** all required fields are validated and tasks are parsed

- **Given** a task missing required fields
- **When** loaded
- **Then** error returned with specific missing field names

### Workspace Isolation

- **Given** a task with setup files
- **When** task workspace is created
- **Then** all setup files are copied to isolated directory

- **Given** previous task completed
- **When** next task starts
- **Then** workspace is completely fresh with no carryover

### Verification

- **Given** task completes with CompletionPromise
- **When** verification runs
- **Then** verification.command executes in task workspace

- **Given** verification command returns non-zero
- **When** results are recorded
- **Then** task marked as `verification_passed: false`

### Metrics

- **Given** task completes
- **When** metrics are collected
- **Then** all fields from LoopState are captured

- **Given** task has expected_iterations
- **When** results are output
- **Then** iteration_delta is calculated

## Directory Structure

```
ralph-orchestrator-2.0/
├── bench/
│   ├── tasks.json              # Task definitions
│   └── tasks/
│       ├── hello-world/
│       │   └── PROMPT.md
│       ├── fizzbuzz-tdd/
│       │   ├── PROMPT.md
│       │   └── test_fizzbuzz.py
│       └── fix-failing-tests/
│           ├── PROMPT.md
│           ├── src/
│           └── tests/
└── specs/
    ├── benchmark-harness.spec.md
    └── benchmark-tasks.spec.md
```

## Non-Goals

- No language-specific task runners (pytest, cargo test, etc. invoked via bash)
- No parallel task execution (sequential for determinism)
- No external API dependencies in starter tasks
- No tasks requiring network access

## Implementation Order

1. **Phase 1**: Define TaskDefinition struct in `ralph-core`
2. **Phase 2**: Implement workspace isolation and setup
3. **Phase 3**: Create starter task suite (3-5 tasks)
4. **Phase 4**: Implement verification runner
5. **Phase 5**: Add metrics collection and JSON output
