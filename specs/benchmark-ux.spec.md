---
status: review
gap_analysis: 2026-01-14
related:
  - benchmark-harness.spec.md
  - terminal-ui.spec.md
---

# Benchmark UX Testing Specification

## Overview

Extension to `benchmark-harness.spec.md` for capturing and replaying terminal output. Enables visual regression testing, replay-based debugging, and prepares extensibility for the future TUI (`ralph-tui`).

This spec follows the principle of being a "thin coordination layer" — capture is decoupled from rendering via the observer pattern, and format interoperability is achieved through export converters rather than native complexity.

## Goals

1. **Record terminal output** with ANSI escape sequences preserved
2. **Replay for visual verification** with timing control
3. **Enable snapshot testing** for CLI output (text and ANSI modes)
4. **Prepare TUI extensibility** via trait abstraction
5. **Ensure workspace isolation** so benchmarks don't pollute the repo

## Non-Goals

- Video export (GIF/MP4) — use VHS/asciinema export instead
- Real-time streaming preview or websocket endpoints
- Cross-terminal emulation (replay to same terminal type)
- Widget-level state diffing (full frame capture only)

## Recording Format

### UX Event Types

Extend the existing JSONL recording format with UX-specific events:

| Event Type | Description |
|------------|-------------|
| `ux.terminal.write` | Raw bytes written to stdout/stderr |
| `ux.terminal.resize` | Terminal dimension change |
| `ux.terminal.color_mode` | Color mode detection result |
| `ux.tui.frame` | TUI frame capture (future) |

### Terminal Write Event

```jsonl
{
  "ts": 1704067200100,
  "event": "ux.terminal.write",
  "data": {
    "bytes": "SGVsbG8sIFdvcmxkIQ==",
    "stdout": true,
    "offset_ms": 100
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `bytes` | string | Base64-encoded raw bytes (preserves ANSI sequences) |
| `stdout` | bool | `true` for stdout, `false` for stderr |
| `offset_ms` | u64 | Milliseconds since session start |

### Terminal Resize Event

```jsonl
{
  "ts": 1704067200500,
  "event": "ux.terminal.resize",
  "data": {
    "width": 120,
    "height": 30,
    "offset_ms": 500
  }
}
```

### Color Mode Event

```jsonl
{
  "ts": 1704067200050,
  "event": "ux.terminal.color_mode",
  "data": {
    "mode": "auto",
    "detected": "always",
    "offset_ms": 50
  }
}
```

### TUI Frame Event (Future)

```jsonl
{
  "ts": 1704067201000,
  "event": "ux.tui.frame",
  "data": {
    "frame_id": 1,
    "width": 80,
    "height": 24,
    "cells": "...",
    "offset_ms": 1000
  }
}
```

Reserved for future `ralph-tui` integration. Frame buffer stored as array of styled cells.

### Session Metadata Extension

Add UX mode to session metadata:

```jsonl
{"ts":1704067200000,"event":"_meta.loop_start","data":{"prompt_file":"PROMPT.md","ux_mode":"cli"}}
{"ts":1704067250000,"event":"_meta.termination","data":{"reason":"CompletionPromise","ux_writes":42,"ux_frames":0}}
```

## Workspace Isolation

### Directory Structure

Each benchmark task runs in an isolated temporary directory:

```
/tmp/ralph-bench-{task-name}-{timestamp}/
├── .git/              # Isolated git repo (NOT main repo)
├── PROMPT.md          # Copied from task prompt_file
├── .agent/
│   └── scratchpad.md  # Fresh scratchpad
└── {setup.files}      # Copied from task definition
```

### Git Isolation

- Temporary workspace initializes its **own** `.git` directory
- Agent commits stay within the temp workspace
- Main repository is never modified during benchmarks

### Cleanup Policy

Rotation-based cleanup with configurable retention:

```json
{
  "cleanup": {
    "policy": "rotate",
    "keep_last_n": 5
  }
}
```

| Policy | Behavior |
|--------|----------|
| `rotate` | Keep last N workspaces, delete older ones |
| `on_success` | Delete on success, keep failures for debugging |
| `always` | Delete immediately after verification |
| `never` | Keep all workspaces (manual cleanup) |

### Recording Directory Structure

```
ralph-orchestrator-2.0/
├── bench/
│   ├── .gitignore         # Ignore recordings/ and results/
│   ├── recordings/        # Session JSONL files (gitignored)
│   ├── results/           # Metrics JSON output (gitignored)
│   └── tasks/             # Task definitions (checked in)
│       ├── tasks.json
│       └── hello-world/
│           └── PROMPT.md
└── specs/
    ├── benchmark-harness.spec.md
    ├── benchmark-tasks.spec.md
    └── benchmark-ux.spec.md
```

### Gitignore Content

```gitignore
# bench/.gitignore
recordings/
results/
*.jsonl
```

## CLI Interface

### Recording with UX Capture

```bash
# Record session with terminal output capture
ralph-bench run tasks.json --record session.jsonl --record-ux

# Record to directory (one file per task)
ralph-bench run tasks.json --record-dir ./recordings/ --record-ux
```

### Replay Modes

```bash
# Replay with terminal output (timing preserved)
ralph-bench replay session.jsonl --ux-mode terminal

# Replay at 2x speed
ralph-bench replay session.jsonl --ux-mode terminal --speed 2x

# Step through events manually
ralph-bench replay session.jsonl --ux-mode terminal --step

# Output plain text (ANSI stripped)
ralph-bench replay session.jsonl --ux-mode text
```

| Mode | Description |
|------|-------------|
| `terminal` | Re-render to stdout with timing and colors |
| `text` | Strip ANSI, output plain text |
| `step` | Pause after each event, press Enter to continue |

### Export Formats

```bash
# Export to asciinema format
ralph-bench export session.jsonl --format cast -o demo.cast

# Export to VHS tape format (for CI golden files)
ralph-bench export session.jsonl --format vhs -o demo.tape

# Export to SVG (static)
ralph-bench export session.jsonl --format svg -o demo.svg
```

| Format | Extension | Use Case |
|--------|-----------|----------|
| `cast` | `.cast` | asciinema web player, sharing |
| `vhs` | `.tape` | CI golden file testing |
| `svg` | `.svg` | Static documentation |

## Snapshot Testing

### Text Snapshots (ANSI Stripped)

For testing output content without styling:

```rust
#[test]
fn test_termination_box_content() {
    let state = LoopState { iteration: 5, ..Default::default() };
    let capture = capture_termination_output(
        &TerminationReason::CompletionPromise,
        &state,
        true,
    );

    insta::assert_snapshot!(capture.text());
}
```

Expected snapshot:

```
┌──────────────────────────────────────────────────────────┐
│ ✓ Loop terminated: Completion promise detected
├──────────────────────────────────────────────────────────┤
│   Iterations:  5
│   Elapsed:     0.0s
└──────────────────────────────────────────────────────────┘
```

### ANSI Snapshots (Full Fidelity)

For testing exact color codes:

```rust
#[test]
fn test_termination_box_colors() {
    let capture = capture_termination_output(...);

    insta::assert_snapshot!(capture.ansi_escaped());
}
```

Expected snapshot (ANSI codes escaped for visibility):

```
\x1b[1m┌──────────────────────────────────────────────────────────┐\x1b[0m
\x1b[1m│\x1b[0m \x1b[32m\x1b[1m✓\x1b[0m Loop terminated: \x1b[32mCompletion promise detected\x1b[0m
```

### Workflow

```bash
# Run snapshot tests
cargo test --features ux-snapshots

# Update snapshots after visual verification
UPDATE_SNAPSHOTS=1 cargo test --features ux-snapshots

# Review changes
git diff tests/snapshots/
```

## TUI Extensibility

### FrameCapture Trait

Abstract interface for capturing rendered output in any display mode:

```rust
pub trait FrameCapture: Send + Sync {
    /// Called after each render to capture the frame.
    async fn capture_frame(&mut self, frame_id: u64, offset_ms: u64);

    /// Returns the captured data for serialization.
    fn take_captures(&mut self) -> Vec<UxEvent>;
}
```

### CLI Mode: CliCapture

Wraps a `Write` implementation to capture bytes:

```rust
pub struct CliCapture<W: Write> {
    inner: W,
    captures: Vec<UxEvent>,
    start_time: Instant,
}

impl<W: Write> Write for CliCapture<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.captures.push(UxEvent::TerminalWrite(TerminalWrite {
            bytes: buf[..n].to_vec(),
            stdout: true,
            offset_ms: self.start_time.elapsed().as_millis() as u64,
        }));
        Ok(n)
    }
}
```

### TUI Mode: TuiCapture (Future)

Captures ratatui frame buffers:

```rust
pub struct TuiCapture {
    captures: Vec<TuiFrame>,
    frame_counter: u64,
    start_time: Instant,
}

impl TuiCapture {
    pub fn capture(&mut self, backend: &TestBackend) {
        let buffer = backend.buffer();
        self.captures.push(TuiFrame {
            frame_id: self.frame_counter,
            buffer: convert_ratatui_buffer(buffer),
            offset_ms: self.start_time.elapsed().as_millis() as u64,
        });
        self.frame_counter += 1;
    }
}
```

### Integration Points

| Mode | Capture Implementation | Output |
|------|------------------------|--------|
| CLI | `CliCapture<W>` wraps stdout | `ux.terminal.write` events |
| TUI | `TuiCapture` wraps TestBackend | `ux.tui.frame` events |

Both produce the same `UxEvent` format, enabling unified replay and export.

## Replay Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Replay Architecture                             │
│                                                                      │
│  ┌────────────────┐     ┌─────────────────┐     ┌────────────────┐  │
│  │ session.jsonl  │────▶│  UxReplayer     │────▶│  Terminal/TUI  │  │
│  └────────────────┘     │                 │     └────────────────┘  │
│                         │  - timing ctrl  │                         │
│                         │  - filter events│                         │
│                         │  - speed mult   │                         │
│                         └─────────────────┘                         │
└─────────────────────────────────────────────────────────────────────┘
```

### UxReplayer

```rust
pub struct UxReplayer {
    events: Vec<TimestampedUxEvent>,
    speed_multiplier: f32,
    current_index: usize,
}

impl UxReplayer {
    pub async fn play_cli<W: Write>(&mut self, writer: &mut W) -> Result<()>;
    pub async fn play_tui<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()>;
}
```

## Format Interoperability

### Asciicast Format (asciinema)

```json
{"version": 2, "width": 80, "height": 24, "timestamp": 1704067200}
[0.5, "o", "Hello, World!"]
[1.0, "o", "\u001b[32m✓\u001b[0m Done"]
```

Conversion from `ux.terminal.write`:
- `offset_ms / 1000.0` → timestamp
- `"o"` for stdout, `"e"` for stderr
- Base64 decode bytes → string

### VHS Tape Format

```tape
Output demo.gif
Set Width 120
Set Height 30

Type "ralph-bench run tasks.json"
Enter
Sleep 5s
```

Export generates VHS-compatible tape files for CI golden testing.

## Acceptance Criteria

### Recording

- **Given** `--record-ux` flag provided
- **When** benchmark runs with terminal output
- **Then** `ux.terminal.write` events are recorded with base64-encoded ANSI sequences

- **Given** terminal is resized during recording
- **When** resize detected
- **Then** `ux.terminal.resize` event is recorded with new dimensions

### Replay

- **Given** recorded session with UX events
- **When** `ralph-bench replay session.jsonl --ux-mode terminal` executes
- **Then** terminal output is replayed with original timing and colors

- **Given** `--speed 2x` flag
- **When** replay executes
- **Then** inter-event delays are halved

- **Given** `--ux-mode text` flag
- **When** replay executes
- **Then** output is plain text with ANSI codes stripped

### Workspace Isolation

- **Given** benchmark task runs
- **When** agent creates files and commits
- **Then** main repository is unaffected (no new commits, no modified files)

- **Given** task workspace created
- **When** git commands execute in workspace
- **Then** commands operate on isolated `.git`, not main repo

### Cleanup

- **Given** `keep_last_n: 5` configured
- **When** 6th benchmark completes
- **Then** oldest workspace is deleted, 5 most recent remain

- **Given** cleanup policy `on_success`
- **When** task verification fails
- **Then** workspace is preserved for debugging

### Export

- **Given** recorded session with UX events
- **When** `ralph-bench export session.jsonl --format cast` executes
- **Then** valid asciinema `.cast` file is produced

- **Given** exported `.cast` file
- **When** loaded in asciinema-player
- **Then** playback matches original terminal output

### Snapshot Testing

- **Given** CLI output capture
- **When** `capture.text()` is called
- **Then** returns string with ANSI codes stripped

- **Given** CLI output capture
- **When** `capture.ansi_escaped()` is called
- **Then** returns string with ANSI codes visible as `\x1b[...]`

## Crate Placement

| Component | Crate | Rationale |
|-----------|-------|-----------|
| `TerminalWrite`, `TuiFrame`, `UxEvent` | `ralph-proto` | Shared types, serializable |
| `FrameCapture` trait | `ralph-proto` | Interface contract |
| `CliCapture<W>` | `ralph-core` | Core recording logic |
| `UxReplayer` | `ralph-core` | Core replay logic |
| `TuiCapture` | `ralph-tui` | TUI-specific implementation |
| `--record-ux`, `--ux-mode` flags | `ralph-bench` | Benchmark CLI interface |
| Workspace isolation logic | `ralph-core` | Shared by harness |

## Implementation Order

1. **Phase 1**: Add `ux_event.rs` with UX event types to `ralph-proto`
2. **Phase 2**: Implement `CliCapture<W>` wrapper in `ralph-core`
3. **Phase 3**: Add `--record-ux` flag to `ralph-bench run`
4. **Phase 4**: Implement `UxReplayer` with `--ux-mode` support
5. **Phase 5**: Add snapshot testing utilities and `insta` integration
6. **Phase 6**: Implement export converters (asciinema, VHS)
7. **Phase 7**: Add `TuiCapture` stub in `ralph-tui` (when TUI implemented)

## References

- [VHS by Charmbracelet](https://github.com/charmbracelet/vhs) - Declarative terminal recording
- [asciinema](https://github.com/asciinema/asciinema) - Terminal session recorder (Rust 3.x)
- [insta](https://github.com/mitsuhiko/insta) - Snapshot testing for Rust
- `benchmark-harness.spec.md` - Parent specification for recording infrastructure
- `benchmark-tasks.spec.md` - Task definition format and workspace isolation
