# Ralph Orchestrator

[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)](https://www.rust-lang.org/)
[![Alpha](https://img.shields.io/badge/status-alpha-yellow)]()

A hat-based multi-agent orchestration framework that puts AI agents in a loop until the task is done.

> "Me fail English? That's unpossible!" - Ralph Wiggum

**Alpha Notice:** Ralph v2 is under active development. It works today, but expect rough edges and breaking changes between releases.

## What is Ralph?

Ralph implements the [Ralph Wiggum technique](https://ghuntley.com/ralph/) — autonomous task completion through continuous AI agent iteration. Unlike simple loops, Ralph v2 introduces **hat-based orchestration**: specialized agent roles (planner, builder, reviewer) that coordinate through events.

> "The orchestrator is a thin coordination layer, not a platform. Agents are smart; let them do the work."

See [AGENTS.md](AGENTS.md) for the full Ralph Tenets and philosophy.

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- At least one AI CLI: [Claude Code](https://github.com/anthropics/claude-code), [Kiro](https://kiro.dev/), or [Gemini CLI](https://github.com/google-gemini/gemini-cli)

### From Source

```bash
git clone https://github.com/mikeyobrien/ralph-orchestrator.git
cd ralph-orchestrator
cargo build --release

# Add to PATH or run directly
./target/release/ralph --help
```

## Quick Start

### 1. Initialize a project

```bash
# Minimal config for Claude
ralph init --backend claude

# Use a preset workflow
ralph init --preset tdd-red-green

# Use a preset with a different backend
ralph init --preset tdd-red-green --backend kiro

# See all available presets
ralph init --list-presets
```

This creates `ralph.yml` in your current directory.

### 2. Define your task

Create `PROMPT.md` with your task description:

```bash
echo "Build a REST API with user authentication" > PROMPT.md
```

Or use an inline prompt when running:

```bash
ralph run -p "Build a REST API with user authentication"
```

### 3. Run Ralph

```bash
# Autonomous mode (headless, default)
ralph run

# Interactive TUI mode (experimental, requires config flag)
ralph run -i

# With a different config
ralph run -c my-config.yml
```

> **Note:** Interactive TUI mode (`-i`) is experimental. Enable it by adding `cli.experimental_tui: true` to your config file.

Ralph iterates through hats (planner → builder → reviewer) until the task completes or limits are reached.

## Key Concepts

### Hats

Specialized agent roles that take turns executing. Each hat has:
- **Triggers**: Events that activate it
- **Instructions**: What to do when active
- **Publishes**: Events it can emit

Default hats: `planner` and `builder`. See presets for multi-hat workflows.

### Events

Hats communicate through events:

```xml
<event topic="plan.complete">Implementation plan ready</event>
<event topic="build.blocked">Need clarification on auth strategy</event>
```

View event history: `ralph events`

### Presets

Pre-configured workflows in `presets/`:

| Category | Examples |
|----------|----------|
| Development | `feature.yml`, `tdd-red-green.yml`, `spec-driven.yml` |
| Debugging | `debug.yml`, `incident-response.yml` |
| Review | `review.yml`, `adversarial-review.yml` |
| Docs | `docs.yml`, `documentation-first.yml` |

### Scratchpad

All hats share `.agent/scratchpad.md` — persistent context across iterations. This enables hats to build on previous work rather than starting fresh.

## CLI Reference

| Command | Description |
|---------|-------------|
| `ralph init --backend <name>` | Initialize with minimal config for backend |
| `ralph init --preset <name>` | Initialize from embedded preset |
| `ralph init --list-presets` | List available presets |
| `ralph run` | Run orchestration loop (default) |
| `ralph run -i` | Interactive TUI mode (experimental, requires `cli.experimental_tui: true`) |
| `ralph run -a` | Autonomous/headless mode |
| `ralph resume` | Continue from existing scratchpad |
| `ralph events` | View event history |

| Flag | Description |
|------|-------------|
| `-p, --prompt-text` | Inline prompt |
| `-c, --config` | Config file (default: `ralph.yml`) |
| `--max-iterations` | Iteration limit (default: 100) |
| `--idle-timeout` | TUI idle timeout in seconds (default: 30) |
| `--completion-promise` | Stop trigger phrase (default: `LOOP_COMPLETE`) |
| `--dry-run` | Show what would execute |
| `-v` / `-q` | Verbose / quiet output |

Run `ralph --help` for full options.

## Project Structure

```
ralph-orchestrator/
├── crates/
│   ├── ralph-cli/       # CLI binary
│   ├── ralph-core/      # Event loop, config, state
│   ├── ralph-adapters/  # AI agent adapters (Claude, Kiro, Gemini)
│   ├── ralph-tui/       # Terminal UI (ratatui)
│   ├── ralph-proto/     # Shared types and traits
│   └── ralph-bench/     # Benchmarking harness
├── presets/             # Workflow configurations
├── ralph.yml            # Default config
└── AGENTS.md            # Philosophy and tenets
```

## Building & Testing

```bash
# Build
cargo build

# Run all tests (includes smoke tests with JSONL replay)
cargo test

# Run smoke tests specifically
cargo test -p ralph-core smoke_runner
```

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure `cargo test` passes
5. Open a Pull Request

## License

MIT License — See [LICENSE](LICENSE) for details.

## Acknowledgments

- **[Geoffrey Huntley](https://ghuntley.com/ralph/)** — Creator of the Ralph Wiggum technique
- **[Harper Reed](https://harper.blog/)** — Spec-driven development methodology

---

*"I'm learnding!" - Ralph Wiggum*
