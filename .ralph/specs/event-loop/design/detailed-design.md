# Hatless Ralph: Event Loop Redesign

## Overview

This document specifies the redesign of Ralph's event loop to be more resilient and extensible through the introduction of "Hatless Ralph" — a constant, irreplaceable coordinator that orchestrates a team of optional hats.

### Problem Statement

The current implementation has critical brittleness:

1. **Planner hat can be overwritten** — Users can misconfigure presets, breaking the event graph
2. **Orphaned events dead-end** — Events with no subscriber terminate the loop unexpectedly
3. **XML event parsing is fragile** — Agents forget to include event tags, causing stalls
4. **No universal fallback** — If the planner hat doesn't exist, recovery fails silently

### Solution: Hatless Ralph

Ralph becomes a constant sovereign coordinator:

- **Always present** — Cannot be replaced or configured away
- **Universal fallback** — Catches all unhandled events
- **Owns completion** — Only Ralph can output `LOOP_COMPLETE`
- **Delegates or executes** — Routes work to hats or does it himself

---

## Detailed Requirements

### Core Architecture

| Requirement | Description |
|-------------|-------------|
| Hatless Ralph is constant | Cannot be replaced, overwritten, or configured away |
| Hats are optional team members | User-defined via config, Ralph coordinates them |
| Ralph runs when no hat triggered | Universal fallback for orphaned events |
| Ralph owns scratchpad | Creates, maintains `.agent/scratchpad.md` |
| Ralph owns completion | Only Ralph outputs `LOOP_COMPLETE` |
| No veto power | Direct hat-to-hat pub/sub bypasses Ralph |

### Event System

| Requirement | Description |
|-------------|-------------|
| JSONL on disk | Events written to `.agent/events.jsonl`, not parsed from output |
| Default publishes | Each hat can specify `default_publishes` as fallback |
| Routing priority | 1) Explicit event → 2) Default event → 3) Ralph fallback |
| Single completion promise | Always `LOOP_COMPLETE`, no per-preset variants |

### Per-Hat Backend Configuration

| Requirement | Description |
|-------------|-------------|
| Named backends | `backend: "claude"` — simple shorthand |
| Kiro agents | `backend: { type: "kiro", agent: "builder" }` — custom MCP/tools |
| Custom backends | `backend: { command: "...", args: [...] }` — any CLI tool |
| Inheritance | Hat without `backend` inherits from `cli.backend` |

### KISS Constraints

| Constraint | Description |
|------------|-------------|
| One iteration = one invocation | No change from current model |
| Sequential hats | No parallel delegation |
| Hats fixed at startup | No dynamic hat lifecycle |
| Single executor | Switch modes per invocation, don't maintain multiple |

---

## Architecture Overview

### System Context

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           RALPH ORCHESTRATOR                             │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                        👑 HATLESS RALPH                            │ │
│  │                                                                    │ │
│  │  • Always present (cannot be configured away)                      │ │
│  │  • Owns scratchpad (.agent/scratchpad.md)                         │ │
│  │  • Owns completion (LOOP_COMPLETE)                                │ │
│  │  • Universal fallback for unhandled events                        │ │
│  │  • Delegates to hats or executes directly                         │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                    │                                     │
│                    ┌───────────────┼───────────────┐                     │
│                    │ delegates     │ delegates     │ delegates           │
│                    ▼               ▼               ▼                     │
│  ┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐         │
│  │   🔨 Builder     │ │   👀 Reviewer    │ │   🔍 Researcher  │         │
│  │   backend:claude │ │   backend:gemini │ │   backend:kiro   │         │
│  └────────┬─────────┘ └────────┬─────────┘ └────────┬─────────┘         │
│           │                    │                    │                    │
│           └────────event───────┴────────event───────┘                    │
│                    (hat-to-hat direct, bypasses Ralph)                   │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                     📁 DISK STATE                                  │ │
│  │  • .agent/scratchpad.md  — Plan, tasks, state                     │ │
│  │  • .agent/events.jsonl   — Event log (source of truth for routing)│ │
│  └────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

### Event Flow

```
                              ┌─────────────┐
                              │ task.start  │
                              └──────┬──────┘
                                     │
                                     ▼
                    ┌────────────────────────────────┐
                    │ Event Router                   │
                    │                                │
                    │ 1. Read .agent/events.jsonl    │
                    │ 2. Find hat with matching      │
                    │    trigger                     │
                    │ 3. If no match → Ralph         │
                    └────────────────┬───────────────┘
                                     │
                    ┌────────────────┼────────────────┐
                    │                │                │
            hat found         hat found         no subscriber
                    │                │                │
                    ▼                ▼                ▼
            ┌───────────┐    ┌───────────┐    ┌───────────┐
            │  Builder  │    │  Reviewer │    │   Ralph   │
            └─────┬─────┘    └─────┬─────┘    └─────┬─────┘
                  │                │                │
                  │                │                │
                  ▼                ▼                ▼
            ┌─────────────────────────────────────────────┐
            │ Agent writes to .agent/events.jsonl        │
            │                                            │
            │ If no event written:                       │
            │   → Use hat's default_publishes            │
            │   → Or fall through to Ralph               │
            └─────────────────────────────────────────────┘
                                     │
                                     ▼
                              ┌─────────────┐
                              │ Next event  │
                              │ or complete │
                              └─────────────┘
```

### Iteration Lifecycle

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ITERATION LIFECYCLE                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. READ STATE                                                           │
│     ├─► Read .agent/scratchpad.md                                       │
│     ├─► Read .agent/events.jsonl (get pending events)                   │
│     └─► Determine which hat/Ralph should run                            │
│                                                                          │
│  2. SELECT EXECUTOR                                                      │
│     ├─► If event has subscriber → Select that hat's backend             │
│     └─► If no subscriber → Select Ralph's backend (cli.backend)         │
│                                                                          │
│  3. BUILD PROMPT                                                         │
│     ├─► If Ralph: build_hatless_ralph() + hat topology (if hats exist)  │
│     └─► If hat: build_hat_prompt() with hat instructions                │
│                                                                          │
│  4. EXECUTE                                                              │
│     ├─► Invoke backend (claude/kiro/gemini/codex/amp/custom)            │
│     └─► Agent reads scratchpad, does work, writes events                │
│                                                                          │
│  5. PROCESS RESULTS                                                      │
│     ├─► Read new events from .agent/events.jsonl                        │
│     ├─► If no events + hat has default_publishes → Use default          │
│     ├─► If no events + no default → Event falls to Ralph                │
│     └─► Route next event or check completion                            │
│                                                                          │
│  6. CHECK COMPLETION                                                     │
│     ├─► Only Ralph can output LOOP_COMPLETE                             │
│     ├─► All tasks [x] or [~]? → Complete                                │
│     └─► Otherwise → Next iteration                                      │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Components and Interfaces

### 1. HatlessRalph (New)

The core coordinator, always present.

```rust
/// Hatless Ralph - the constant coordinator
pub struct HatlessRalph {
    /// Backend for Ralph's own execution
    backend: CliBackend,

    /// Hat topology for prompt injection
    hat_topology: Option<HatTopology>,
}

impl HatlessRalph {
    /// Build Ralph's prompt
    pub fn build_prompt(&self, context: &IterationContext) -> String {
        let mut prompt = self.core_prompt();

        if let Some(topology) = &self.hat_topology {
            prompt.push_str(&self.multi_hat_section(topology));
        } else {
            prompt.push_str(&self.solo_mode_section());
        }

        prompt
    }

    /// Check if Ralph should handle this event (no hat claimed it)
    pub fn should_handle(&self, event: &Event, registry: &HatRegistry) -> bool {
        !registry.has_subscriber(&event.topic)
    }
}
```

### 2. HatRegistry (Modified)

No longer creates default planner/builder. Just holds user-defined hats.

```rust
pub struct HatRegistry {
    hats: HashMap<HatId, Hat>,
}

impl HatRegistry {
    /// Create from config - NO default hats
    pub fn from_config(config: &RalphConfig) -> Self {
        let mut registry = Self::new();

        // Only user-defined hats, no defaults
        for (id, hat_config) in &config.hats {
            registry.register(Self::hat_from_config(id, hat_config));
        }

        registry
    }

    /// Check if any hat subscribes to this topic
    pub fn has_subscriber(&self, topic: &Topic) -> bool {
        self.hats.values().any(|h| h.triggers_on(topic))
    }

    /// Get hat for topic, if any
    pub fn get_for_topic(&self, topic: &Topic) -> Option<&Hat> {
        self.hats.values().find(|h| h.triggers_on(topic))
    }
}
```

### 3. EventReader (New)

Reads events from JSONL file instead of parsing XML from output.

```rust
pub struct EventReader {
    path: PathBuf,
    last_position: u64,
}

impl EventReader {
    pub fn new(path: PathBuf) -> Self {
        Self { path, last_position: 0 }
    }

    /// Read new events since last read
    pub fn read_new_events(&mut self) -> Result<Vec<Event>> {
        let file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(self.last_position))?;

        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for line in reader.lines() {
            let event: Event = serde_json::from_str(&line?)?;
            events.push(event);
        }

        self.last_position = file.stream_position()?;
        Ok(events)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub topic: String,
    pub payload: Option<String>,
    pub ts: DateTime<Utc>,
}
```

### 4. HatBackend (New)

Per-hat backend configuration.

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum HatBackend {
    /// Named backend: "claude", "kiro", "gemini", "codex", "amp"
    Named(String),

    /// Kiro with custom agent
    KiroAgent {
        #[serde(rename = "type")]
        backend_type: String,  // must be "kiro"
        agent: String,
        #[serde(default)]
        args: Option<Vec<String>>,
    },

    /// Fully custom backend
    Custom {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default = "default_prompt_mode")]
        prompt_mode: String,
        prompt_flag: Option<String>,
    },
}

impl HatBackend {
    /// Resolve to a CliBackend
    pub fn to_cli_backend(&self, default: &CliBackend) -> CliBackend {
        match self {
            HatBackend::Named(name) => CliBackend::from_name(name),
            HatBackend::KiroAgent { agent, args, .. } => {
                CliBackend::kiro_with_agent(agent, args.as_deref())
            }
            HatBackend::Custom { command, args, prompt_mode, prompt_flag } => {
                CliBackend::custom(command, args, prompt_mode, prompt_flag.as_deref())
            }
        }
    }
}
```

### 5. EventLoop (Modified)

Main orchestration loop, now with Ralph as constant.

```rust
pub struct EventLoop {
    ralph: HatlessRalph,
    registry: HatRegistry,
    event_reader: EventReader,
    config: RalphConfig,
}

impl EventLoop {
    pub fn new(config: RalphConfig) -> Self {
        let registry = HatRegistry::from_config(&config);
        let ralph = HatlessRalph::new(&config, &registry);
        let event_reader = EventReader::new(".agent/events.jsonl".into());

        Self { ralph, registry, event_reader, config }
    }

    /// Run one iteration
    pub fn iterate(&mut self) -> Result<IterationResult> {
        // 1. Read new events
        let events = self.event_reader.read_new_events()?;

        // 2. Determine who runs
        let (executor, prompt) = if let Some(event) = events.last() {
            if let Some(hat) = self.registry.get_for_topic(&event.topic) {
                // Hat handles this event
                let backend = hat.backend.to_cli_backend(&self.default_backend());
                let prompt = self.build_hat_prompt(hat, event);
                (backend, prompt)
            } else {
                // No subscriber - Ralph handles it
                let prompt = self.ralph.build_prompt(&self.context());
                (self.ralph.backend.clone(), prompt)
            }
        } else {
            // No events - Ralph runs (initial or stalled)
            let prompt = self.ralph.build_prompt(&self.context());
            (self.ralph.backend.clone(), prompt)
        };

        // 3. Execute
        let result = executor.invoke(&prompt)?;

        // 4. Process results
        self.process_iteration_result(result)
    }

    /// Process result - check for default_publishes if no event written
    fn process_iteration_result(&mut self, result: ExecutionResult) -> Result<IterationResult> {
        let new_events = self.event_reader.read_new_events()?;

        if new_events.is_empty() {
            // Check if current hat has default_publishes
            if let Some(hat) = &self.current_hat {
                if let Some(default) = &hat.default_publishes {
                    // Inject default event
                    self.write_event(Event {
                        topic: default.clone(),
                        payload: Some("Default event (no explicit event written)".into()),
                        ts: Utc::now(),
                    })?;
                }
            }
            // If still no event, falls through to Ralph next iteration
        }

        // Check completion (only from Ralph)
        if self.is_ralph_iteration && result.output.contains("LOOP_COMPLETE") {
            return Ok(IterationResult::Complete);
        }

        Ok(IterationResult::Continue)
    }
}
```

### 6. InstructionBuilder (Modified)

New method for hatless Ralph prompt.

```rust
impl InstructionBuilder {
    /// Build prompt for hatless Ralph
    pub fn build_hatless_ralph(&self, context: &Context) -> String {
        let mut prompt = String::new();

        // Core identity
        prompt.push_str(RALPH_CORE_PROMPT);

        // Conditional: solo or multi-hat mode
        if context.has_hats() {
            prompt.push_str(&self.multi_hat_section(context.hat_topology()));
        } else {
            prompt.push_str(SOLO_MODE_SECTION);
        }

        // Inject current state
        prompt.push_str(&self.state_section(context));

        prompt
    }
}

const RALPH_CORE_PROMPT: &str = r#"
I'm Ralph. Fresh context each iteration.

### 0a. ORIENTATION
Study `{specs_dir}` to understand requirements.
Don't assume features aren't implemented—search first.

### 0b. SCRATCHPAD
Study `{scratchpad}`. It's shared state. It's memory.

Task markers:
- `[ ]` pending
- `[x]` done
- `[~]` cancelled (with reason)

### GUARDRAILS
{guardrails}
"#;

const WORKFLOW_SECTION: &str = r#"
## WORKFLOW

### 1. GAP ANALYSIS
Compare specs against codebase. Use parallel subagents (up to 10) for searches.

### 2. PLAN
Update `{scratchpad}` with prioritized tasks.

### 3. IMPLEMENT
Pick ONE task. Only 1 subagent for build/tests.

### 4. COMMIT
Capture the why, not just the what. Mark `[x]` in scratchpad.

### 5. REPEAT
Until all tasks `[x]` or `[~]`.
"#;

const HATS_SECTION: &str = r#"
## HATS

Delegate via events.

{hat_topology_table}
"#;

const CUSTOM_HAT_PROMPT: &str = r#"
You are {hat_name}. Fresh context each iteration.

### 0. ORIENTATION
Study the incoming event context.
Don't assume work isn't done—verify first.

### 1. EXECUTE
{derived_behaviors}
Only 1 subagent for build/tests.

### 2. REPORT
Publish result event with evidence.

### GUARDRAILS
{guardrails}
"#;

const EVENT_WRITING_SECTION: &str = r#"
## EVENTS
Write events to `.agent/events.jsonl` as JSONL:
{"topic": "build.task", "payload": "...", "ts": "2026-01-14T12:00:00Z"}
"#;

const DONE_SECTION: &str = r#"
## DONE
All tasks `[x]` or `[~]`? Output: {completion_promise}
"#;
```

---

## Data Models

### Config Schema

```yaml
# ralph.yml

cli:
  backend: "claude"  # Default backend for Ralph and hats that don't specify

event_loop:
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 100
  # terminal_events: []  # Future: user-declared terminal events

hats:
  builder:
    name: "Builder"
    triggers: ["build.task"]
    publishes: ["build.done", "build.blocked"]
    default_publishes: "build.done"  # NEW: fallback if no event written
    backend: "claude"  # NEW: per-hat backend
    instructions: |
      ...

  researcher:
    name: "Researcher"
    triggers: ["research.task"]
    publishes: ["research.finding"]
    backend:
      type: "kiro"
      agent: "researcher"  # Uses .kiro/agents/researcher.json
    instructions: |
      ...

  reviewer:
    name: "Reviewer"
    triggers: ["review.request"]
    publishes: ["review.approved", "review.changes_requested"]
    backend: "gemini"  # Different perspective
    instructions: |
      ...
```

### Rust Types

```rust
#[derive(Debug, Deserialize)]
pub struct RalphConfig {
    pub cli: CliConfig,
    pub event_loop: EventLoopConfig,
    #[serde(default)]
    pub hats: HashMap<String, HatConfig>,
}

#[derive(Debug, Deserialize)]
pub struct HatConfig {
    pub name: String,
    pub triggers: Vec<String>,
    #[serde(default)]
    pub publishes: Vec<String>,
    #[serde(default)]
    pub default_publishes: Option<String>,  // NEW
    #[serde(default)]
    pub backend: Option<HatBackend>,  // NEW
    #[serde(default)]
    pub instructions: String,
}

#[derive(Debug, Deserialize)]
pub struct EventLoopConfig {
    #[serde(default = "default_completion_promise")]
    pub completion_promise: String,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
}
```

### Event Format

```jsonl
{"topic": "task.start", "payload": "Implement user authentication", "ts": "2024-01-15T10:00:00Z"}
{"topic": "build.task", "payload": "Implement auth endpoint", "ts": "2024-01-15T10:05:00Z"}
{"topic": "build.done", "payload": "Auth endpoint complete, tests pass", "ts": "2024-01-15T10:30:00Z"}
```

---

## Error Handling

### Validation Errors (Config Load Time)

| Error | When | Recovery |
|-------|------|----------|
| Orphan event | Hat publishes event with no subscriber | Error with fix suggestion |
| Unreachable hat | No event path leads to hat | Error with event graph |
| Ambiguous trigger | Two hats trigger on same event | Error listing both hats |
| Invalid backend | Unknown backend name | Error with valid options |
| Invalid Kiro agent | Agent file doesn't exist | Error with path suggestion |

### Runtime Errors

| Error | When | Recovery |
|-------|------|----------|
| No events written | Hat completes without writing event | Use `default_publishes` or fall to Ralph |
| Backend unavailable | CLI tool not in PATH | Error with install instructions |
| Events file corrupt | Invalid JSON in events.jsonl | Truncate to last valid line, warn |
| Scratchpad missing | `.agent/scratchpad.md` doesn't exist | Ralph creates it |

### Safeguards (Unchanged)

- Max iterations (default: 100)
- Stall detection (same state N times)
- Cost limits (if configured)

---

## Testing Strategy

### Unit Tests

| Component | Tests |
|-----------|-------|
| `HatlessRalph` | Prompt building (solo vs multi-hat), completion detection |
| `HatRegistry` | No default hats, subscriber lookup, topology generation |
| `EventReader` | JSONL parsing, incremental reading, corrupt file handling |
| `HatBackend` | Named resolution, Kiro agent building, custom backend |
| `InstructionBuilder` | Prompt assembly, hat topology injection |

### Integration Tests

| Test | Description |
|------|-------------|
| Solo mode completion | Ralph alone, no hats, completes successfully |
| Multi-hat delegation | Ralph → hat → Ralph closes |
| Hat-to-hat direct | Builder → Reviewer bypasses Ralph |
| Orphaned event fallback | Unknown event falls through to Ralph |
| Default publishes | Hat forgets event, default fires |
| Mixed backends | Claude builder, Kiro researcher, Gemini reviewer |

### E2E Scenario Tests

Scripted scenarios with mock CLI backend.

```yaml
# test-scenarios/solo-mode-complete.yml
name: "Solo mode completes successfully"
config:
  hats: {}  # No hats

iterations:
  - hat: ralph
    writes:
      scratchpad: |
        ## Tasks
        - [ ] Implement feature
      events: []

  - hat: ralph
    writes:
      scratchpad: |
        ## Tasks
        - [x] Implement feature
      events: []
    output: "LOOP_COMPLETE"

expect:
  completion: true
  iterations: 2
```

```yaml
# test-scenarios/orphaned-event-fallback.yml
name: "Orphaned event falls through to Ralph"
config:
  hats:
    builder:
      triggers: ["build.task"]
      publishes: ["build.done"]

iterations:
  - hat: ralph
    writes:
      events:
        - { topic: "build.task", payload: "Implement auth" }

  - hat: builder
    writes:
      events:
        - { topic: "unknown.event", payload: "Something unexpected" }

  - hat: ralph  # Falls through because no subscriber for unknown.event
    writes:
      scratchpad: |
        ## Tasks
        - [x] Implement auth
      events: []
    output: "LOOP_COMPLETE"

expect:
  completion: true
  iterations: 3
```

### Mock CLI Backend

```rust
pub struct MockCliBackend {
    scenario: Scenario,
    iteration: usize,
}

impl MockCliBackend {
    pub fn invoke(&mut self, _prompt: &str) -> Result<ExecutionResult> {
        let step = &self.scenario.iterations[self.iteration];
        self.iteration += 1;

        // Write scripted files
        if let Some(scratchpad) = &step.writes.scratchpad {
            fs::write(".agent/scratchpad.md", scratchpad)?;
        }

        for event in &step.writes.events {
            self.append_event(event)?;
        }

        Ok(ExecutionResult {
            output: step.output.clone().unwrap_or_default(),
            exit_code: 0,
        })
    }
}
```

---

## Appendices

### A. Technology Choices

| Choice | Rationale |
|--------|-----------|
| JSONL for events | Structured, appendable, easy to debug (`cat`, `jq`) |
| Per-hat backends | Leverage each tool's strengths (Claude for coding, Kiro for MCP) |
| Single executor | KISS — hats run sequentially, no need for parallel executors |
| Mock CLI for tests | Deterministic, fast, no API keys needed |

### B. Research Findings

See `research/` directory:
- `current-implementation.md` — Analysis of existing code
- `per-hat-backends.md` — Full backend flexibility design + adapter docs

### C. Alternative Approaches Considered

| Approach | Why Not |
|----------|---------|
| Ralph intercepts all events | Violates "Let Ralph Ralph" — too prescriptive |
| Parallel hat execution | KISS — adds complexity without clear benefit |
| Dynamic hat loading | KISS — hats fixed at startup is simpler |
| XML event parsing (current) | Fragile — agents forget tags, regex is unreliable |

### D. Migration Notes

**Breaking changes:**
- `planner` hat no longer exists — Ralph IS the planner
- Event parsing from output deprecated — use `.agent/events.jsonl`
- Per-preset completion promises removed — always `LOOP_COMPLETE`

**Migration path:**
1. Remove `planner` hat from custom configs
2. Add `default_publishes` to hats that might forget events
3. Update hat instructions to write JSONL events
4. Test with `ralph validate` before running

### E. Adapter Documentation Links

| Backend | Documentation |
|---------|---------------|
| **Claude** | [Docs](https://docs.anthropic.com/en/docs/claude-code/overview) · [GitHub](https://github.com/anthropics/claude-code) · [Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices) |
| **Kiro** | [Docs](https://kiro.dev/docs/cli/) · [Custom Agents](https://kiro.dev/docs/cli/custom-agents/) · [Config Reference](https://kiro.dev/docs/cli/custom-agents/configuration-reference/) |
| **Gemini** | [Docs](https://developers.google.com/gemini-code-assist/docs/gemini-cli) · [GitHub](https://github.com/google-gemini/gemini-cli) |
| **Codex** | [Docs](https://developers.openai.com/codex/cli/) · [GitHub](https://github.com/openai/codex) |
| **Amp** | [Manual](https://ampcode.com/manual) · [Guides](https://github.com/sourcegraph/amp-examples-and-guides) |
