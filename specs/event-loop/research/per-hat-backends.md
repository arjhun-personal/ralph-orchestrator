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

## Full Backend Flexibility

Support **any backend** with **any configuration** per hat — mixing Claude, Gemini, Kiro agents, and custom commands in the same team.

### The Full Matrix

```yaml
cli:
  backend: "claude"  # Default for Ralph (hatless)

hats:
  # Direct Claude - best for coding
  builder:
    triggers: ["build.task"]
    backend: "claude"

  # Kiro with custom agent - has AWS MCP tools
  infra:
    triggers: ["infra.task"]
    backend:
      type: "kiro"
      agent: "infra-admin"

  # Kiro with different agent - has Confluence MCP
  researcher:
    triggers: ["research.task"]
    backend:
      type: "kiro"
      agent: "researcher"

  # Gemini - different perspective for review
  reviewer:
    triggers: ["review.request"]
    backend: "gemini"

  # Custom command - internal tool
  compliance:
    triggers: ["compliance.check"]
    backend:
      command: "internal-compliance-agent"
      args: ["--strict"]
      prompt_mode: "stdin"
```

### Backend Types Summary

| Type | Config Syntax | Invocation |
|------|---------------|------------|
| **Named** | `backend: "claude"` | `claude --dangerously-skip-permissions` |
| **Kiro (default)** | `backend: "kiro"` | `kiro-cli chat --no-interactive --trust-all-tools` |
| **Kiro (agent)** | `backend: { type: "kiro", agent: "builder" }` | `kiro-cli --agent builder --no-interactive --trust-all-tools` |
| **Gemini** | `backend: "gemini"` | `gemini --yolo -p` |
| **Codex** | `backend: "codex"` | `codex exec --full-auto` |
| **Amp** | `backend: "amp"` | `amp --dangerously-allow-all -x` |
| **Custom** | `backend: { command: "...", args: [...] }` | Whatever you specify |

### Why Mix Backends?

| Scenario | Best Backend | Why |
|----------|--------------|-----|
| Complex coding | Claude | Best at reasoning, long context |
| AWS infrastructure | Kiro + agent | Native AWS MCP tools |
| Quick research | Gemini | Fast, good at summarization |
| Code review | Different model | Fresh perspective catches different issues |
| Internal tools | Custom | Integrate proprietary agents |
| Cost optimization | Haiku via Kiro | Cheaper for simple tasks |

### Config Schema

```rust
pub enum HatBackend {
    /// Named backend (claude, kiro, gemini, codex, amp)
    Named(String),

    /// Kiro with custom agent
    KiroAgent {
        agent: String,
        /// Optional: override default kiro args
        args: Option<Vec<String>>,
    },

    /// Fully custom backend
    Custom {
        command: String,
        args: Vec<String>,
        prompt_mode: PromptMode,
        prompt_flag: Option<String>,
    },
}
```

### YAML Syntax Options

```yaml
# Option 1: String shorthand for named backends
backend: "claude"

# Option 2: Object for Kiro agents
backend:
  type: "kiro"
  agent: "builder"

# Option 3: Object for custom
backend:
  command: "my-agent"
  args: ["--headless"]
  prompt_mode: "stdin"

# Option 4: Kiro default (no agent)
backend:
  type: "kiro"
  # No agent = default kiro behavior
```

### Execution Flow

```
Hat triggered
    │
    ├─► Get hat's backend config
    │       │
    │       ├─► Named ("claude") → Use standard CliBackend::claude()
    │       │
    │       ├─► Kiro agent → kiro-cli --agent <name> ...
    │       │
    │       ├─► Named ("gemini") → Use standard CliBackend::gemini()
    │       │
    │       └─► Custom → Build command from config
    │
    ├─► Execute via appropriate executor (PTY or process)
    │
    └─► Read events from .agent/events.jsonl
```

### Executor Management

With mixed backends, need to handle different execution modes:

| Backend | Execution Mode | Notes |
|---------|---------------|-------|
| Claude | PTY | Rich TUI, interactive |
| Kiro | Process | Headless, no TUI |
| Gemini | Process | Headless |
| Custom | Configurable | Depends on tool |

**Options:**
1. **Single executor, switch modes** — Simpler, one at a time
2. **Per-backend executors** — More complex, but cleaner separation
3. **Lazy executor creation** — Create on first use of that backend type

**Recommendation:** Option 1 (single executor) for KISS. Hats run sequentially anyway.

---

## Adapter Documentation Resources

Reference documentation for each supported backend to assist with per-hat implementation.

### Claude (Anthropic)

| Resource | URL |
|----------|-----|
| **Official Docs** | https://docs.anthropic.com/en/docs/claude-code/overview |
| **GitHub** | https://github.com/anthropics/claude-code |
| **Best Practices** | https://www.anthropic.com/engineering/claude-code-best-practices |
| **Settings Reference** | `~/.claude/settings.json`, `.claude/settings.json` |

**Key flags:**
- `--dangerously-skip-permissions` — Skip approval prompts (autonomous mode)
- Prompt mode: stdin (interactive TUI preserved)

**Configuration files:**
- `CLAUDE.md` — Project context auto-loaded into conversations
- Settings hierarchy: user → project → local project

---

### Kiro (AWS)

| Resource | URL |
|----------|-----|
| **Official Docs** | https://kiro.dev/docs/cli/ |
| **Custom Agents** | https://kiro.dev/docs/cli/custom-agents/ |
| **Agent Config Reference** | https://kiro.dev/docs/cli/custom-agents/configuration-reference/ |
| **Subagents** | https://kiro.dev/docs/cli/chat/subagents/ |
| **Migration from Q** | https://kiro.dev/docs/cli/migrating-from-q/ |

**Key flags:**
- `--agent <name>` — Use custom agent configuration
- `--no-interactive` — Headless mode (exits on Ctrl+C)
- `--trust-all-tools` — Allow all tools without prompting

**Configuration files:**
- `.kiro/agents/*.json` — Local agent definitions
- `~/.kiro/agents/*.json` — Global agent definitions
- `~/.kiro/settings/mcp.json` — MCP server config

**Agent config fields:**
```json
{
  "name": "builder",
  "prompt": "...",
  "model": "claude-sonnet-4",
  "tools": ["read", "write", "shell", "@builtin"],
  "allowedTools": ["read", "write"],
  "mcpServers": { ... }
}
```

---

### Gemini (Google)

| Resource | URL |
|----------|-----|
| **Official Docs** | https://developers.google.com/gemini-code-assist/docs/gemini-cli |
| **GitHub** | https://github.com/google-gemini/gemini-cli |
| **Google AI Studio** | https://aistudio.google.com |
| **Gemini API Docs** | https://ai.google.dev/gemini-api/docs |

**Key flags:**
- `--yolo` — Dangerous mode (skip approvals)
- `-p <prompt>` — Pass prompt as argument

**Key features:**
- Free tier: 60 req/min, 1000 req/day with personal Google account
- Access to Gemini 2.5 Pro with 1M token context
- Uses ReAct loop with built-in tools and MCP servers
- Fully open source (Apache 2.0)

---

### Codex (OpenAI)

| Resource | URL |
|----------|-----|
| **Official Docs** | https://developers.openai.com/codex/cli/ |
| **CLI Reference** | https://developers.openai.com/codex/cli/reference/ |
| **CLI Features** | https://developers.openai.com/codex/cli/features/ |
| **Quickstart** | https://developers.openai.com/codex/quickstart/ |
| **GitHub** | https://github.com/openai/codex |
| **Models** | https://developers.openai.com/codex/models/ |

**Key flags:**
- `exec` (or `e`) — Scripted/CI mode, non-interactive
- `--full-auto` — Run without human interaction
- `--dangerously-bypass-approvals-and-sandbox` (or `--yolo`) — Skip all approvals

**Configuration:**
- `~/.codex/config.toml` — Global config
- `-c key=value` — Override config for single invocation
- MCP servers configurable via `codex mcp` commands

**Key features:**
- Default model: gpt-5-codex (macOS/Linux), gpt-5 (Windows)
- Image support (PNG, JPEG) for design specs
- Built in Rust for speed

---

### Amp (Sourcegraph)

| Resource | URL |
|----------|-----|
| **Owner's Manual** | https://ampcode.com/manual |
| **Examples & Guides** | https://github.com/sourcegraph/amp-examples-and-guides |
| **CLI Guide** | https://github.com/sourcegraph/amp-examples-and-guides/blob/main/guides/cli/README.md |
| **NPM Package** | https://www.npmjs.com/package/@sourcegraph/amp |

**Key flags:**
- `-x` or `--execute` — Execute mode (send message, wait, exit)
- `--dangerously-allow-all` — Allow all tools without approval

**Agent modes:**
- `smart` — State-of-the-art models (Claude Opus 4.5, GPT-5.1)
- `rush` — Fast/efficient models (Claude Haiku 4.5)

**Configuration:**
- `AGENT.md` — Project context file (like CLAUDE.md)
- `AMP_API_KEY` — Environment variable for API key
- `AMP_SETTINGS_FILE` — Custom settings location

**Key features:**
- Threads sync to ampcode.com across devices
- Command allowlisting for security
- MCP server support
- $10 daily free grant for new users

---

## Quick Reference: Backend Invocations

| Backend | Autonomous Invocation |
|---------|----------------------|
| **Claude** | `claude --dangerously-skip-permissions` (stdin) |
| **Kiro** | `kiro-cli chat --no-interactive --trust-all-tools "prompt"` |
| **Kiro + agent** | `kiro-cli --agent builder --no-interactive --trust-all-tools "prompt"` |
| **Gemini** | `gemini --yolo -p "prompt"` |
| **Codex** | `codex exec --full-auto "prompt"` |
| **Amp** | `amp --dangerously-allow-all -x "prompt"` |
