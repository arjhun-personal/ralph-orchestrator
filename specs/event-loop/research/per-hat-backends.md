# Per-Hat Backend Configuration Research

Exploring how hats can be tied to specific agent configurations (e.g., Kiro agents).

---

## Current State

**Global backend configuration:**
```yaml
cli:
  backend: "claude"  # All hats use this
```

All hats share the same CLI backend. This is limiting because different agents have different strengths.

## Proposed: Per-Hat Backend

Allow each hat to specify its own backend:

```yaml
cli:
  backend: "claude"  # Default for Ralph and hats that don't specify

hats:
  builder:
    name: "Builder"
    triggers: ["build.task"]
    backend: "claude"       # Explicit: Claude is great at coding

  researcher:
    name: "Researcher"
    triggers: ["research.task"]
    backend: "kiro"         # Kiro has MCP tools for AWS/internal systems

  reviewer:
    name: "Reviewer"
    triggers: ["review.request"]
    backend: "gemini"       # Different perspective, good at catching issues
```

## Use Cases

| Use Case | Why Different Backends Help |
|----------|----------------------------|
| **AWS Infrastructure** | Kiro has built-in AWS MCP tools |
| **Code Review** | Different model = different perspective |
| **Research/Exploration** | Kiro can access internal wikis via MCP |
| **High-stakes coding** | Use most capable model (Claude Opus) |
| **Cost optimization** | Use cheaper model for simple tasks |

## Design Considerations

### 1. Hatless Ralph's Backend

| Option | Description |
|--------|-------------|
| **A: Config default** | Ralph uses `cli.backend` (current behavior) |
| **B: Always Claude** | Ralph is hardcoded to Claude for consistency |
| **C: Configurable** | New `ralph.backend` field |

**Recommendation:** Option A (config default) — keeps it simple, user controls Ralph's backend via existing config.

### 2. Inheritance

```yaml
cli:
  backend: "claude"  # Default

hats:
  builder:
    # backend not specified → inherits "claude"
    triggers: ["build.task"]

  researcher:
    backend: "kiro"  # Override for this hat
    triggers: ["research.task"]
```

### 3. Custom Backend Per Hat

For full flexibility, allow custom backend config per hat:

```yaml
hats:
  infrastructure:
    name: "Infrastructure"
    triggers: ["infra.task"]
    backend:
      command: "kiro-cli"
      args: ["chat", "--profile", "prod-admin", "--trust-all-tools"]
      prompt_mode: "arg"
```

This allows passing different profiles, flags, or even entirely custom commands per hat.

### 4. Executor Lifecycle

Currently, one executor is created per orchestrator. With per-hat backends:

```
┌─────────────────────────────────────────────────────────────────┐
│                     ORCHESTRATOR                                 │
│                                                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ Ralph Executor  │  │ Builder Executor │  │ Researcher Exec │  │
│  │ (claude)        │  │ (claude)         │  │ (kiro)          │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

Options:
- **Lazy creation:** Create executor on first use of that backend
- **Eager creation:** Create all executors at startup
- **Pooling:** Reuse executors for same backend across hats

### 5. PTY vs Non-PTY

| Backend | Execution Mode |
|---------|---------------|
| Claude | PTY (interactive TUI) |
| Kiro | Process (headless) |
| Gemini | Process (headless) |

Mixed backends require handling both execution modes in the same run.

## Implementation Sketch

### Config Schema Change

```rust
pub struct HatConfig {
    pub name: String,
    pub triggers: Vec<String>,
    pub publishes: Vec<String>,
    pub instructions: String,
    pub default_publishes: Option<String>,

    // NEW: Per-hat backend
    pub backend: Option<HatBackendConfig>,
}

pub enum HatBackendConfig {
    /// Use a known backend by name
    Named(String),  // "claude", "kiro", "gemini"

    /// Custom backend configuration
    Custom {
        command: String,
        args: Vec<String>,
        prompt_mode: String,
        prompt_flag: Option<String>,
    },
}
```

### Executor Resolution

```rust
impl EventLoop {
    fn get_executor_for_hat(&self, hat: &Hat) -> &dyn Executor {
        match &hat.backend {
            Some(backend) => self.executors.get(backend),
            None => &self.default_executor,
        }
    }
}
```

## Questions for Clarification

1. **Should Ralph (hatless) have a separate backend config?**
   - Or always use the global `cli.backend`?

2. **Should we support inline custom backends per hat?**
   - Or just allow referencing named backends?

3. **How do we handle mixed PTY/non-PTY in the same run?**
   - TUI only for PTY backends, plain output for others?

4. **Should backend changes trigger validation warnings?**
   - E.g., "builder uses kiro which may have different capabilities"

## Kiro Subagent Integration

Kiro CLI supports **custom agents** that can be invoked with `--agent <name>`. This maps beautifully to Ralph's hat concept.

### Kiro Agent Files

Agents are JSON files stored in:
- **Local:** `.kiro/agents/` (project-specific)
- **Global:** `~/.kiro/agents/` (user-wide)

### Agent Configuration Structure

```json
// .kiro/agents/builder.json
{
  "name": "builder",
  "description": "Implements code following existing patterns",
  "prompt": "You are a builder. Implement one task at a time...",
  "model": "claude-sonnet-4",
  "tools": ["read", "write", "shell", "@builtin"],
  "allowedTools": ["read", "write", "shell"],
  "mcpServers": {
    "github": {
      "command": "gh-mcp",
      "args": ["--repo", "myorg/myrepo"]
    }
  }
}
```

### Key Configuration Options

| Field | Purpose |
|-------|---------|
| `prompt` | High-level context (can use `file://` for external) |
| `model` | Which Claude model to use |
| `tools` | Available tools (`@builtin`, `@server_name`, etc.) |
| `allowedTools` | Tools usable without user prompting (glob patterns) |
| `mcpServers` | MCP server configurations |
| `resources` | File paths the agent can access |
| `hooks` | Lifecycle event handlers |

### Invoking a Kiro Agent

```bash
kiro-cli --agent builder "Implement the auth endpoint"
```

### Mapping to Ralph Hats

```yaml
# ralph.yml
hats:
  builder:
    triggers: ["build.task"]
    backend:
      type: "kiro"
      agent: "builder"  # → invokes `kiro-cli --agent builder`

  researcher:
    triggers: ["research.task"]
    backend:
      type: "kiro"
      agent: "researcher"  # → invokes `kiro-cli --agent researcher`

  reviewer:
    triggers: ["review.request"]
    backend:
      type: "claude"  # Uses Claude directly (no Kiro agent)
```

### What This Enables

| Capability | How |
|------------|-----|
| **Per-hat MCP servers** | Each Kiro agent has its own `mcpServers` |
| **Per-hat models** | builder uses Sonnet, researcher uses Haiku |
| **Per-hat tool permissions** | Restrict builder to write, researcher to read-only |
| **Per-hat prompts** | Agent-level system prompts in Kiro config |
| **Per-hat resources** | Scope file access per agent |

### Flow

```
Ralph (hatless) receives event
    │
    ├─► Decides to delegate to builder hat
    │
    ├─► Looks up builder's backend config
    │       │
    │       └─► type: "kiro", agent: "builder"
    │
    ├─► Invokes: kiro-cli --agent builder --no-interactive --trust-all-tools "prompt"
    │
    └─► Reads events from .agent/events.jsonl
```

### Kiro Agent Files as Code

Since Kiro agents are JSON files in `.kiro/agents/`, they can be:
- Checked into the repo (version controlled)
- Shared via presets
- Generated dynamically by Ralph

This means hat configurations can include both Ralph config AND Kiro agent definitions.

## Updated Recommendation

Support three backend modes:

1. **Named backend** (simple)
   ```yaml
   backend: "claude"
   ```

2. **Kiro agent** (powerful)
   ```yaml
   backend:
     type: "kiro"
     agent: "builder"
   ```

3. **Custom inline** (full flexibility)
   ```yaml
   backend:
     command: "my-custom-agent"
     args: ["--mode", "headless"]
     prompt_mode: "stdin"
   ```
