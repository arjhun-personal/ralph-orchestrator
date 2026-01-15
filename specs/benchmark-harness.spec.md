---
status: implemented
gap_analysis: 2026-01-14
related:
  - benchmark-tasks.spec.md
  - benchmark-ux.spec.md
---

# Benchmark Harness Specification

## Overview

Implement a testing harness to evaluate and benchmark Ralph orchestration loops. The harness records agent sessions by subscribing to `EventBus::publish()` (the same event stream used for iteration routing), then supports replay for analysis and regression testing.

## Goals

1. **Record sessions** by observing the EventBus - no polling, pure event subscription
2. **Replay sessions** for debugging, analysis, and demonstration
3. **Collect metrics** from `LoopState`: iterations, duration, termination reason, cost
4. **Batch benchmarking** for comparing prompt strategies or model performance

## Concepts

### Recording Architecture

Record by subscribing to `EventBus::publish()` - the same event stream used for iteration:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Recording Architecture                               │
│                                                                          │
│  ┌─────────────┐         ┌─────────────────┐                            │
│  │  EventLoop  │────────▶│    EventBus     │                            │
│  └─────────────┘         │   publish()     │                            │
│                          └────────┬────────┘                            │
│                                   │                                      │
│                    ┌──────────────┼──────────────┐                      │
│                    ▼              ▼              ▼                      │
│             ┌───────────┐  ┌───────────┐  ┌───────────────┐            │
│             │  Hat A    │  │  Hat B    │  │  Observer     │            │
│             │(subscriber)│  │(subscriber)│  │  (recorder)   │            │
│             └───────────┘  └───────────┘  └───────┬───────┘            │
│                                                   │                      │
│                                                   ▼                      │
│                                          ┌───────────────┐              │
│                                          │  session.jsonl │              │
│                                          └───────────────┘              │
└─────────────────────────────────────────────────────────────────────────┘
```

### Session Metrics

Captured from existing `LoopState` struct:
- `iteration`: Current iteration number (1-indexed)
- `consecutive_failures`: Failure count
- `cumulative_cost`: Total USD spent
- `elapsed()`: Duration since start

Termination captured via `TerminationReason` enum:
- `CompletionPromise`, `MaxIterations`, `MaxRuntime`, `MaxCost`, `ConsecutiveFailures`, `Stopped`

## CLI Interface

Separate `ralph-bench` binary for benchmarking (keeps `ralph` focused on orchestration):

```bash
# Run with recording
ralph-bench run tasks.json --record ./recordings/session-001.jsonl

# Replay a recorded session
ralph-bench replay ./recordings/session-001.jsonl --speed 2x

# Batch run multiple tasks
ralph-bench run tasks.json --record-dir ./recordings/ --output results.json
```

### CLI Arguments

| Argument | Type | Description |
|----------|------|-------------|
| `run <tasks>` | subcommand | Run benchmark tasks |
| `replay <session>` | subcommand | Replay recorded session |
| `--record <path>` | PathBuf | Record session to JSONL file |
| `--record-dir <dir>` | PathBuf | Record each task to separate file |
| `--output <path>` | PathBuf | Write metrics summary to JSON |
| `--speed <multiplier>` | f32 | Replay speed (default: 1.0) |
| `--step` | bool | Manual step-through replay |

## Recording Format

### Event Records

Events serialized directly from `ralph_proto::Event` (already implements `Serialize`):

```jsonl
{"ts":1704067200000,"event":"bus.publish","data":{"topic":"task.start","payload":"Implement fizzbuzz","source":null,"target":"default"}}
{"ts":1704067201000,"event":"bus.publish","data":{"topic":"task.continue","payload":"Continue with the task","source":null,"target":null}}
```

### Metadata Records

Loop state snapshots (prefixed with `_meta`):

```jsonl
{"ts":1704067200000,"event":"_meta.loop_start","data":{"prompt_file":"PROMPT.md","max_iterations":100}}
{"ts":1704067215000,"event":"_meta.iteration","data":{"n":1,"elapsed_ms":15000,"hat":"default"}}
{"ts":1704067250000,"event":"_meta.termination","data":{"reason":"CompletionPromise","iterations":3,"elapsed_secs":50.0}}
```

### CLI Output Records

Execution results from `CliExecutor`:

```jsonl
{"ts":1704067210000,"event":"cli.output","data":{"hat":"default","success":true,"exit_code":0,"output_preview":"..."}}
```

## Task Definition Format

```json
{
  "tasks": [
    {
      "name": "hello-world",
      "prompt_file": "prompts/hello.md",
      "completion_promise": "TASK_COMPLETE",
      "max_iterations": 10,
      "verification": "python hello.py | grep -q 'Hello, World!'"
    },
    {
      "name": "fizzbuzz-tdd",
      "prompt_file": "prompts/fizzbuzz.md",
      "completion_promise": "TESTS_PASSING",
      "max_iterations": 20,
      "verification": "pytest test_fizzbuzz.py -q"
    }
  ]
}
```

## Implementation

### Option A: Observer Pattern (Preferred)

Add observer callback to `EventBus`:

```rust
// ralph-proto/src/event_bus.rs
impl EventBus {
    /// Sets an observer that receives all published events.
    pub fn set_observer<F>(&mut self, observer: F)
    where
        F: Fn(&Event) + Send + 'static,
    {
        self.observer = Some(Box::new(observer));
    }

    pub fn publish(&mut self, event: Event) -> Vec<HatId> {
        // Notify observer before routing
        if let Some(ref observer) = self.observer {
            observer(&event);
        }
        // ... existing routing logic
    }
}
```

### Option B: Wrapper Pattern

Wrap EventBus without modifying proto crate:

```rust
// ralph-core/src/recording.rs
pub struct RecordingEventBus<W: Write> {
    inner: EventBus,
    sink: W,
}

impl<W: Write> RecordingEventBus<W> {
    pub fn publish(&mut self, event: Event) -> Vec<HatId> {
        let record = serde_json::json!({
            "ts": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
            "event": "bus.publish",
            "data": event,
        });
        writeln!(self.sink, "{}", record).ok();
        self.inner.publish(event)
    }
}
```

## Acceptance Criteria

### Recording

- **Given** `--record ./session.jsonl` flag provided
- **When** benchmark runs
- **Then** observer subscribes to `EventBus::publish()` and logs all events to JSONL

- **Given** recording is active
- **When** `EventLoop::process_output()` publishes events
- **Then** each event is logged with timestamp before routing to subscribers

- **Given** recording session completes
- **When** `TerminationReason` is determined
- **Then** final `_meta.termination` record written with loop state snapshot

### Replay

- **Given** valid session.jsonl file
- **When** `ralph-bench replay session.jsonl` executes
- **Then** events are output in recorded order with timestamps displayed

- **Given** `--speed 2x` flag
- **When** replay executes
- **Then** inter-event delays are halved

- **Given** `--step` flag
- **When** replay executes
- **Then** playback pauses after each event until Enter is pressed

### Batch Benchmarking

- **Given** tasks.json with multiple task definitions
- **When** `ralph-bench run tasks.json` executes
- **Then** each task runs sequentially in isolated working directory

- **Given** `--output results.json` flag
- **When** batch completes
- **Then** aggregated metrics written: task_name, iterations, duration_secs, termination_reason, success

### Verification

- **Given** task has `verification` command
- **When** Ralph outputs completion promise
- **Then** verification command runs to confirm actual success

- **Given** verification command fails (non-zero exit)
- **When** results are recorded
- **Then** task marked as `verified: false` despite promise detection

## Crate Placement

| Component | Crate |
|-----------|-------|
| Observer trait/callback on EventBus | `ralph-proto` |
| RecordingEventBus wrapper (if Option B) | `ralph-core` |
| BenchmarkRunner, TaskDefinition | `ralph-core` |
| SessionRecorder, SessionPlayer | `ralph-core` |
| `ralph-bench` binary | `ralph-bench` (new crate) |

## Non-Goals

- No web UI or dashboard
- No database storage (JSONL files only)
- No multi-model A/B testing framework
- No real-time streaming metrics endpoint

## Implementation Order

1. **Phase 1**: Add observer callback to `EventBus::publish()` in `ralph-proto`
2. **Phase 2**: Implement `SessionRecorder` that writes JSONL in `ralph-core`
3. **Phase 3**: Implement `SessionPlayer` for replay in `ralph-core`
4. **Phase 4**: Create `ralph-bench` crate with `run` and `replay` subcommands
5. **Phase 5**: Implement batch task runner with verification
6. **Phase 6**: Add workspace isolation and cleanup
