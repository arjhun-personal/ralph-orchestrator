---
status: review
gap_analysis: 2026-01-14
related:
  - v1-v2-feature-parity.spec.md
  - cli-adapters.spec.md
---

# Feature Parity Specification

## Overview

Ensure Rust 2.0 achieves complete feature parity with Python v1.x, using the **identical configuration format**. Users can switch from Python to Rust with zero config changes—only the installation method differs.

## Goals

1. **Identical config format**: `ralph.yml` files work unchanged between v1 and v2
2. **Behavioral equivalence**: Same config produces same orchestration behavior
3. **Clean distribution exit**: Transition from PyPI to native distribution
4. **No translation layer**: v2 natively understands the existing config schema

## Configuration Schema

Rust 2.0 uses the exact same configuration schema as Python v1.x. No field renaming, no structural changes.

### Core Fields

| Field | Type | Description |
|-------|------|-------------|
| `agent` | string | Backend CLI: `claude`, `gemini`, `codex`, `amp`, or `auto` |
| `agent_priority` | list | Fallback order for auto-detection |
| `prompt_file` | string | Path to prompt file |
| `completion_promise` | string | Completion detection string |
| `max_iterations` | integer | Maximum loop iterations |
| `max_runtime` | integer | Maximum runtime in seconds |
| `max_cost` | float | Maximum cost in USD |

### Adapter Settings

| Field | Type | Description |
|-------|------|-------------|
| `adapters.<backend>.timeout` | integer | CLI execution timeout |
| `adapters.<backend>.enabled` | boolean | Include in auto-detection |

### Feature Flags

| Field | Type | v2 Status |
|-------|------|-----------|
| `verbose` | boolean | Supported |
| `archive_prompts` | boolean | Deferred (warn if enabled) |
| `enable_metrics` | boolean | Deferred (warn if enabled) |

### Dropped Fields

These v1 fields are not applicable to CLI-based backends and are ignored with a warning:

| Field | Reason |
|-------|--------|
| `max_tokens` | Token limits controlled by CLI tool |
| `adapters.*.tool_permissions` | CLI tool manages its own permissions |
| `retry_delay` | Retry logic handled differently |

## Distribution Migration

### New Installation Methods

Rust 2.0 is distributed natively, not via PyPI:

| Channel | Command | Platform |
|---------|---------|----------|
| Homebrew | `brew install mikeyobrien/tap/ralph` | macOS, Linux |
| Cargo | `cargo install ralph-orchestrator` | Cross-platform |
| GitHub Releases | Direct binary download | Universal |
| Shell installer | `curl -fsSL https://ralph.dev/install.sh \| sh` | macOS, Linux |

### PyPI Exit Strategy

#### Phase 1: Deprecation Release (v1.3.0)

Final functional Python release that:
- Works normally with no behavior changes
- Prints deprecation notice on every invocation
- Points users to new installation methods

#### Phase 2: Tombstone Release (v1.4.0)

Non-functional Python package that:
- Contains no orchestration code
- Prints migration instructions and exits on any invocation
- Reserves the package name to prevent squatting

Tombstone message content:
- New installation commands for each platform
- Link to migration documentation
- Confirmation that config files remain compatible

#### Phase 3: Yank Old Versions

After 6-12 months, yank pre-tombstone versions from PyPI to prevent accidental installs of deprecated Python version.

### Build Pipeline

Use `cargo-dist` for automated multi-platform releases:

| Target | Runner | Binary |
|--------|--------|--------|
| x86_64-linux | ubuntu-latest | ralph-x86_64-unknown-linux-gnu |
| aarch64-linux | ubuntu-latest (cross) | ralph-aarch64-unknown-linux-gnu |
| x86_64-darwin | macos-13 | ralph-x86_64-apple-darwin |
| aarch64-darwin | macos-14 | ralph-aarch64-apple-darwin |
| x86_64-windows | windows-latest | ralph-x86_64-pc-windows-msvc.exe |

On tag push, GitHub Actions:
1. Cross-compile for all targets
2. Create GitHub Release with binaries
3. Generate shell installer script
4. Update Homebrew tap formula
5. Publish to crates.io

## Auto-Detection (agent: auto)

When config specifies `agent: auto`, implement detection logic:

### Detection Order

1. Check `agent_priority` list if present, test in order
2. If no priority list, use default order: claude, gemini, codex, amp
3. For each candidate, check if binary exists in PATH
4. Use first available backend

### Detection Method

For each backend, run a version check:

| Backend | Detection Command | Success Criteria |
|---------|-------------------|------------------|
| claude | `claude --version` | Exit code 0 |
| gemini | `gemini --version` | Exit code 0 |
| codex | `codex --version` | Exit code 0 |
| amp | `amp --version` | Exit code 0 |

Cache detection result for session duration.

## Deferred Feature Warnings

For features not yet implemented in v2, emit warnings to stderr:

| Field | Warning |
|-------|---------|
| `archive_prompts: true` | Feature not yet available in v2 |
| `enable_metrics: true` | Feature not yet available in v2 |

### Suppressing Warnings

Add config field `_suppress_warnings: true` to silence all warnings (for CI environments).

## Acceptance Criteria

### Config Loading

- **Given** a valid v1.x `ralph.yml` file
- **When** Rust v2 loads the config
- **Then** all fields are parsed correctly with no errors

- **Given** a config with unknown fields
- **When** config is loaded
- **Then** unknown fields are ignored (forward compatibility)

### Core Fields

- **Given** config with `agent: claude`
- **When** config is loaded
- **Then** Claude CLI backend is selected

- **Given** config with `max_runtime: 3600`
- **When** config is loaded
- **Then** max runtime is set to 3600 seconds

### Auto-Detection

- **Given** `agent: auto` and `agent_priority: [gemini, claude]`
- **When** only `claude` is available in PATH
- **Then** Claude backend is selected

- **Given** `agent: auto` and no `agent_priority`
- **When** `claude` and `gemini` are both available
- **Then** `claude` is selected (default priority)

- **Given** `agent: auto`
- **When** no supported backends are available
- **Then** an error is returned listing available backend options

### Deferred Features

- **Given** config with `archive_prompts: true`
- **When** config is loaded
- **Then** a warning is emitted noting this feature is not yet available

- **Given** config with `_suppress_warnings: true`
- **When** config is loaded
- **Then** no warnings are emitted

### Dropped Fields

- **Given** config with `max_tokens: 4096`
- **When** config is loaded
- **Then** field is ignored with a warning

### Distribution (PyPI Exit)

- **Given** user runs `pip install ralph-orchestrator` after v1.4.0
- **When** they invoke `ralph`
- **Then** migration instructions are printed
- **And** the process exits with code 0

- **Given** user has v1.3.0 installed
- **When** they invoke `ralph run`
- **Then** deprecation notice is printed to stderr
- **And** orchestration proceeds normally

## Error Handling

1. **Invalid YAML**: Return parse error with line number
2. **Unknown fields**: Ignore silently (forward compatibility)
3. **Invalid field types**: Return error with expected type
4. **Missing required fields**: Return error listing missing fields
5. **Backend not found** (auto-detect): Return error with installation guidance

## Crate Placement

| Component | Crate |
|-----------|-------|
| RalphConfig struct | `ralph-core` |
| Config parsing, validation | `ralph-core` |
| Auto-detection logic | `ralph-adapters` |
| CLI argument overrides | `ralph-cli` |

## Implementation Order

### Rust Implementation

1. **Phase 1**: Define RalphConfig struct matching v1 schema in `ralph-core`
2. **Phase 2**: Implement YAML parsing with serde
3. **Phase 3**: Add validation and warning system for deferred/dropped fields
4. **Phase 4**: Implement auto-detection in `ralph-adapters`
5. **Phase 5**: Wire config to event loop

### Distribution Setup

1. **Phase A**: Configure `cargo-dist` in workspace Cargo.toml
2. **Phase B**: Create GitHub Actions release workflow
3. **Phase C**: Create Homebrew tap repository
4. **Phase D**: Test cross-platform builds

### PyPI Transition

1. **Phase X**: Release Python v1.3.0 with deprecation warnings
2. **Phase Y**: Release Python v1.4.0 tombstone package
3. **Phase Z**: Yank old Python versions after migration period

## Timeline

| Milestone | Description |
|-----------|-------------|
| v2.0.0-alpha | Core event loop, config parity |
| v2.0.0-beta | cargo-dist setup, Homebrew tap, GitHub releases |
| v1.3.0 (Python) | Deprecation notices, points to v2 install methods |
| v2.0.0 | Stable Rust release |
| v1.4.0 (Python) | Tombstone package |
| v2.0.0 + 6mo | Yank old Python versions |
