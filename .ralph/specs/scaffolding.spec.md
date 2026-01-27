---
status: implemented
gap_analysis: 2026-01-14
---

# Scaffolding Spec

## Goal

Initialize Rust monorepo with Cargo workspace containing five crates (`ralph-core`, `ralph-tui`, `ralph-adapters`, `ralph-cli`, `ralph-proto`) under `./crates/`.

## Context

- Rewrite of Python ralph-orchestrator (v1.x) to Rust
- See `src/ralph_orchestrator/` for v1 module structure reference
- TUI framework: `ratatui` + `crossterm`
- Async runtime: `tokio`

## Requirements

1. Create root `Cargo.toml` with workspace configuration
   - Rust 2024 edition
   - Workspace-level dependencies for shared crates
   - Workspace lints (forbid unsafe, warn clippy::pedantic)

2. Create `crates/` directory with five crates:
   - `ralph-proto`: Shared types, error definitions, traits
   - `ralph-core`: Orchestration loop, config, state management
   - `ralph-adapters`: Agent adapters (Claude, Kiro, Gemini, ACP)
   - `ralph-tui`: Terminal UI using ratatui
   - `ralph-cli`: Binary entry point, CLI argument parsing

3. Each crate must have:
   - `Cargo.toml` inheriting workspace settings
   - `src/lib.rs` (or `src/main.rs` for ralph-cli)
   - Module doc comment describing purpose

4. Dependency graph (arrows = "depends on"):
   ```
   ralph-cli → ralph-tui → ralph-core → ralph-proto
                         ↘ ralph-adapters → ralph-proto
                                          ↘ ralph-core
   ```

5. Include these workspace dependencies:
   - `tokio` (async runtime)
   - `ratatui`, `crossterm` (TUI)
   - `serde`, `serde_json`, `serde_yaml` (serialization)
   - `clap` (CLI parsing)
   - `thiserror`, `anyhow` (errors)
   - `tracing`, `tracing-subscriber` (logging)
   - `async-trait` (async traits)

6. Add `.gitignore` entry for `/target/` if not present

## Acceptance Criteria

**Given** an empty `./crates/` directory
**When** scaffolding is complete
**Then**:
- `cargo check --workspace` succeeds
- `cargo clippy --workspace` reports no errors
- Each crate compiles independently
- Dependency graph matches requirement 4

## Out of Scope

- Implementation of any business logic
- Tests beyond compile verification
- CI/CD configuration
