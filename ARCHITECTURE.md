# Ralph Orchestrator: Complete Architecture Walkthrough

## High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              ralph-cli (Entry Point)                           │
│  Commands: run, plan, task, loops, web, init, events, resume, tools            │
│  └─ loop_runner.rs: run_loop_impl() — drives the main orchestration loop       │
└────────┬──────────────────────────┬──────────────────────────┬──────────────────┘
         │                          │                          │
         ▼                          ▼                          ▼
┌─────────────────┐  ┌──────────────────────┐  ┌──────────────────────────────┐
│  ralph-adapters  │  │     ralph-core        │  │       ralph-tui              │
│                  │  │  (Orchestration       │  │  (Terminal UI - ratatui)     │
│  Claude (PTY)    │  │     Engine)           │  │  Real-time loop monitoring   │
│  Gemini          │  │                       │  └──────────────────────────────┘
│  Codex           │  │  EventLoop            │
│  Pi              │  │  HatlessRalph         │  ┌──────────────────────────────┐
│  Amp             │  │  HatRegistry          │  │     ralph-telegram           │
│  Custom          │  │  EventBus             │  │  Human-in-the-loop (RObot)   │
│                  │  │  EventParser          │  │  Telegram bot for Q&A        │
│  Auto-detection  │  │  MemoryStore          │  └──────────────────────────────┘
│  PTY Executor    │  │  TaskStore            │
└─────────────────┘  │  SkillRegistry        │  ┌──────────────────────────────┐
                     │  Worktree (parallel)   │  │     ralph-proto              │
                     │  MergeQueue            │  │  Shared types & traits       │
                     │  Diagnostics           │  │  Event, EventBus, Hat, Topic │
                     └───────────────────────-┘  └──────────────────────────────┘
```

## The Core Mental Model

Ralph is a **thin coordination layer** that drives AI coding agents (Claude, Gemini, etc.) in a loop. The key insight: **agents are smart; Ralph just steers them with signals, not scripts.**

The entire system is built on three pillars:

1. **Pub/Sub Event Bus** — Hats (agent personas) communicate via topic-based events
2. **Backpressure** — Quality gates (tests, lint, typecheck) reject bad work automatically
3. **Fresh Context** — Each iteration starts with a clean context window, re-reading specs

## Layer-by-Layer Walkthrough

### 1. `ralph-proto` — The Foundation Types

This is the shared protocol crate. Everything else depends on it.

**`Topic`** (`topic.rs`) — Routing keys with glob matching:
```
"build.task"   — exact match
"impl.*"       — wildcard suffix (matches impl.done, impl.started)
"*.done"       — wildcard prefix
"*"            — global wildcard (matches everything)
```

**`Event`** (`event.rs`) — The message unit:
```rust
struct Event {
    topic: Topic,        // Routing key (e.g., "build.done")
    payload: String,     // Free-form content
    source: Option<HatId>,  // Who published it
    target: Option<HatId>,  // Direct handoff (bypasses routing)
}
```

**`Hat`** (`hat.rs`) — Agent personas with pub/sub contracts:
```rust
struct Hat {
    id: HatId,
    name: String,
    subscriptions: Vec<Topic>,  // What events trigger this hat
    publishes: Vec<Topic>,      // What events this hat emits
    instructions: String,       // Injected into prompt
}
```

Default hats define the core workflow:
- **Planner**: subscribes to `task.start`, `build.done`, `build.blocked` → publishes `build.task`
- **Builder**: subscribes to `build.task` → publishes `build.done`, `build.blocked`

**`EventBus`** (`event_bus.rs`) — Central routing hub:
- Hats register with subscriptions
- `publish(event)` routes to matching subscribers with priority: **specific subscriptions > global wildcards**
- `human.*` events go to a separate queue (for RObot/Telegram)
- Observers receive all events (for recording/TUI)

### 2. `ralph-core` — The Orchestration Engine

This is the heart of Ralph.

#### The Event Loop (`event_loop/mod.rs`)

The `EventLoop` struct ties everything together:

```
┌──────────────────────────────────────────────────┐
│                   EventLoop                       │
│                                                   │
│  config: RalphConfig      ← YAML configuration   │
│  registry: HatRegistry    ← All registered hats   │
│  bus: EventBus            ← Pub/sub routing        │
│  state: LoopState         ← Iteration tracking     │
│  ralph: HatlessRalph      ← The constant coord.    │
│  event_reader: EventReader ← JSONL file watcher    │
│  skill_registry            ← Auto-injected skills  │
│  robot_service             ← Telegram (optional)   │
│  diagnostics               ← Logging/debugging     │
└──────────────────────────────────────────────────┘
```

**The iteration cycle** (driven by `loop_runner.rs`):

```
                    ┌─────────────────────────┐
                    │  1. initialize()         │
                    │  Publish task.start      │
                    │  Store objective         │
                    └────────────┬────────────┘
                                 │
          ┌──────────────────────▼──────────────────────┐
          │  2. check_termination()                      │
          │  Max iterations? Max runtime? Max cost?      │
          │  Consecutive failures? Loop thrashing?       │◄──────┐
          │  Stale loop? Stop/restart signal?            │       │
          └──────────────────────┬──────────────────────┘       │
                                 │ (no termination)             │
                    ┌────────────▼────────────┐                 │
                    │  3. next_hat()           │                 │
                    │  Who has pending events? │                 │
                    │  → Always "ralph" in     │                 │
                    │    multi-hat mode        │                 │
                    └────────────┬────────────┘                 │
                                 │                              │
                    ┌────────────▼────────────┐                 │
                    │  4. build_prompt()       │                 │
                    │  Collect ALL pending     │                 │
                    │  events from ALL hats    │                 │
                    │  + scratchpad content    │                 │
                    │  + memories (auto-inject)│                 │
                    │  + ready tasks           │                 │
                    │  + skills                │                 │
                    │  + robot guidance        │                 │
                    └────────────┬────────────┘                 │
                                 │                              │
                    ┌────────────▼────────────┐                 │
                    │  5. Execute via adapter  │                 │
                    │  PTY → claude/gemini/... │                 │
                    │  Agent works, emits      │                 │
                    │  <event> tags in output  │                 │
                    └────────────┬────────────┘                 │
                                 │                              │
                    ┌────────────▼────────────┐                 │
                    │  6. process_output()     │                 │
                    │  Parse <event> tags      │                 │
                    │  Validate backpressure   │                 │
                    │  Route to bus            │                 │
                    │  Check for LOOP_COMPLETE │                 │
                    └────────────┬────────────┘                 │
                                 │                              │
                    ┌────────────▼────────────┐                 │
                    │  7. Read JSONL events    │                 │
                    │  (from ralph emit CLI)   │                 │
                    │  External event injection│                 │
                    └────────────┬────────────┘                 │
                                 │                              │
                                 └──────────────────────────────┘
```

#### Hatless Ralph (`hatless_ralph.rs`)

The philosophical center of the architecture. Key principle: **Ralph is always present and cannot be configured away.**

```
┌─────────────────────────────────────────────────┐
│              HatlessRalph                        │
│                                                  │
│  "The constant coordinator"                      │
│                                                  │
│  • Solo mode (no custom hats):                   │
│    Ralph does everything directly                │
│                                                  │
│  • Multi-hat mode (custom hats defined):         │
│    Custom hats define TOPOLOGY ONLY              │
│    Ralph is the SOLE EXECUTOR                    │
│    Hats inform which instructions get injected   │
│                                                  │
│  Builds prompts with:                            │
│    - Completion promise (LOOP_COMPLETE)           │
│    - Hat topology (## HATS table)                │
│    - Active hat instructions                     │
│    - Objective (persists across iterations)       │
│    - Skill index                                 │
│    - Robot guidance                              │
└─────────────────────────────────────────────────┘
```

The `## HATS` table in the prompt looks like:
```
## HATS
| Hat | Subscribes | Publishes | Description |
|-----|-----------|-----------|-------------|
| planner | task.start, build.done | build.task | Plans tasks |
| builder | build.task | build.done, build.blocked | Implements code |
```

This tells the agent what role to play and what events to emit.

#### Event Parser (`event_parser.rs`)

Agents communicate back by emitting XML-style event tags in their output:

```xml
<event topic="build.done">
tests: pass
lint: pass
typecheck: pass
audit: pass
coverage: pass
complexity: 7
duplication: pass
performance: pass
specs: pass
</event>
```

The `EventParser` extracts these and validates **backpressure evidence**:

```
BackpressureEvidence {
    tests_passed: bool,        ← Required
    lint_passed: bool,         ← Required
    typecheck_passed: bool,    ← Required
    audit_passed: bool,        ← Required
    coverage_passed: bool,     ← Required
    complexity_score: f64,     ← Required (<=10)
    duplication_passed: bool,  ← Required
    performance_regression: Option<bool>,  ← Regression blocks
    mutants: Option<MutationEvidence>,     ← Warning-only
    specs_verified: Option<bool>,          ← Fail blocks
}
```

If `all_passed()` is false, the `build.done` event is **rejected** — this is the backpressure mechanism that prevents agents from declaring victory without proof.

#### Completion Detection

An agent signals completion via `LOOP_COMPLETE` (the "completion promise"). The parser has safety checks:
1. Promise must be the **last non-empty line** in output
2. Promise must NOT appear inside any `<event>` tag payload
3. Required events must have been seen during the loop lifetime
4. In persistent mode, completion is suppressed

#### Memory System (`memory.rs`, `memory_store.rs`)

Persistent learning across sessions stored at `.ralph/agent/memories.md`:

```markdown
## Patterns
### mem-1737372000-a1b2
> Uses barrel exports for all modules
<!-- tags: imports, structure | created: 2025-01-20 -->

## Decisions
### mem-1737372100-c3d4
> Chose Postgres over SQLite for production
<!-- tags: database | created: 2025-01-20 -->
```

Four types: **Pattern**, **Decision**, **Fix**, **Context**

Memories are auto-injected into prompts with a configurable budget (truncation).

#### Task System (`task.rs`, `task_store.rs`)

JSONL-based task tracking at `.ralph/agent/tasks.jsonl`:
- Tasks have statuses: `Open → InProgress → Closed/Failed`
- Priority 1-5 (1 = highest)
- Dependency tracking via `blocked_by`
- Loop ID tagging for multi-loop ownership
- Ready tasks (unblocked + open) are auto-injected into prompts

#### Hat Registry (`hat_registry.rs`)

Manages all registered hats with O(1) prefix indexing for fast topic lookups. Uses `BTreeMap` for deterministic ordering (alphabetical). Key behaviors:
- `get_for_topic()` — find first hat matching a topic
- `can_publish()` — enforce that hats only publish declared topics
- `subscribers()` — find all hats subscribed to a topic

### 3. `ralph-adapters` — Backend Integrations

Supports multiple AI backends through a common interface:

```
┌──────────────────────────────────────────┐
│              CliBackend                    │
│                                           │
│  Translates config into CLI commands:     │
│  • claude → claude --dangerously-skip...  │
│  • gemini → gemini-cli                    │
│  • codex  → codex                         │
│  • pi     → pi-coding-agent              │
│  • amp    → amp                           │
│  • custom → user-defined command          │
│                                           │
│  PtyExecutor (primary execution mode):    │
│  • Spawns agent in a PTY for rich output  │
│  • Supports interactive & observe modes   │
│  • Signal handling (SIGINT→SIGTERM→SIGKILL)│
│  • Streaming output to TUI/console        │
└──────────────────────────────────────────┘
```

Auto-detection (`auto_detect.rs`) finds available backends in `$PATH`.

### 4. `ralph-telegram` — Human-in-the-Loop (RObot)

```
Agent ──human.interact──► EventBus ──► Telegram Bot ──► Human
                                                            │
Agent ◄──human.response──◄ EventBus ◄──────────────────────┘

Human proactive message ──► human.guidance ──► Injected as
                                               "## ROBOT GUIDANCE"
                                               in next prompt
```

The loop **blocks** on `human.interact` until response or timeout.

### 5. Parallel Loops (Worktrees)

```
Main Repo (./)
├── .ralph/loop.lock           ← Primary loop holds this
├── .worktrees/
│   ├── ralph-20250124-a3f2/   ← Worktree loop 1 (git worktree)
│   │   ├── (full repo copy)
│   │   └── .ralph/ → symlinked memories, specs, tasks
│   └── ralph-20250124-b5c6/   ← Worktree loop 2
│       └── ...
├── .ralph/merge-queue.jsonl   ← Event-sourced merge queue
└── .ralph/loops.json          ← Registry of all loops
```

When a worktree loop completes, it queues for merge. The primary loop processes the merge queue.

### 6. The Data Flow — One Complete Iteration

```
                         ralph.yml (config)
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     PROMPT ASSEMBLY                              │
│                                                                  │
│  1. Auto-inject skills (memories data, ralph-tools, robot)      │
│  2. Scratchpad content (agent's working memory, 4K token budget) │
│  3. Ready tasks (unblocked open tasks)                          │
│  4. HatlessRalph prompt:                                        │
│     - System instructions                                        │
│     - ## HATS topology table                                     │
│     - Active hat instructions                                    │
│     - ## EVENTS (pending events with payloads)                   │
│     - ## OBJECTIVE (original user prompt)                         │
│     - Robot guidance (if any)                                    │
│     - Completion promise definition                              │
│     - Skill index                                                │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                     PTY EXECUTION                                │
│                                                                  │
│  claude --dangerously-skip-permissions -p "<assembled prompt>"   │
│                                                                  │
│  Agent works: reads files, writes code, runs tests...            │
│  Agent emits: <event topic="build.done">tests: pass...</event>   │
│  Agent may emit: LOOP_COMPLETE                                   │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                     OUTPUT PROCESSING                            │
│                                                                  │
│  EventParser.parse(output) → Vec<Event>                          │
│                                                                  │
│  For each event:                                                 │
│    • Validate backpressure (build.done requires all checks pass) │
│    • Validate publish permissions (hat can only publish declared) │
│    • Detect loop thrashing (repeated blocked events)             │
│    • Detect stale loops (same topic 3+ times)                    │
│    • Route via EventBus → pending queues for next iteration      │
│                                                                  │
│  Also read JSONL events file for external events (ralph emit)    │
│                                                                  │
│  Check: LOOP_COMPLETE? → Terminate                               │
│  Check: No pending events? → inject_fallback_event(task.resume)  │
└─────────────────────────────────────────────────────────────────┘
```

### 7. Configuration (`config.rs`)

`ralph.yml` supports both v1 flat and v2 nested formats:

```yaml
# v2 format
cli:
  backend: claude          # Which AI to use
event_loop:
  max_iterations: 100      # Safety limit
  max_runtime_seconds: 7200
  max_cost_usd: 50.0
  completion_promise: "LOOP_COMPLETE"
  starting_event: "task.start"
  required_events: []      # Must be seen before LOOP_COMPLETE
  persistent: false        # Keep alive after completion
core:
  scratchpad: ".ralph/scratchpad.md"
  workspace_root: "."
memories:
  enabled: true
  inject: auto
  budget: 2000
tasks:
  enabled: true
hats:
  planner:
    name: "Planner"
    triggers: ["task.start", "build.done"]
    publishes: ["build.task"]
  builder:
    name: "Builder"
    triggers: ["build.task"]
    publishes: ["build.done", "build.blocked"]
skills:
  enabled: true
robot:
  enabled: false
```

### 8. Termination Conditions

The loop can terminate for many reasons, each with a specific exit code:

| Reason | Exit Code | Description |
|--------|-----------|-------------|
| `CompletionPromise` | 0 | Agent said LOOP_COMPLETE |
| `ConsecutiveFailures` | 1 | Too many failures in a row |
| `LoopThrashing` | 1 | Repeated blocked event dispatch |
| `LoopStale` | 1 | Same topic emitted 3+ times |
| `MaxIterations` | 2 | Hit iteration limit |
| `MaxRuntime` | 2 | Hit time limit |
| `MaxCost` | 2 | Hit cost limit |
| `Interrupted` | 130 | SIGINT (Ctrl+C) |
| `RestartRequested` | 3 | Telegram /restart |
| `Cancelled` | 0 | Graceful cancel via loop.cancel |

### 9. The Default Workflow (Planner → Builder cycle)

```
task.start ──► Planner reads specs, creates plan
                  │
                  ▼
              build.task ──► Builder implements code
                                │
                       ┌────────┴────────┐
                       ▼                 ▼
                  build.done        build.blocked
                  (with backpressure    │
                   evidence)            ▼
                       │           Planner simplifies
                       ▼           or unblocks
                  Planner reviews,     │
                  picks next task      └──► build.task
                       │
                       ▼
                  (repeat until all tasks done)
                       │
                       ▼
                  LOOP_COMPLETE
```

This is the essence of Ralph: a **reactive event-driven loop** where agents coordinate through topic-based events, quality is enforced through backpressure gates, and the orchestrator stays thin — steering with signals rather than prescribing steps.
