---
status: completed
created: 2026-01-20
started: 2026-01-20
completed: 2026-01-20
---
# Task: Rename `resume` Subcommand to `--continue` Flag

## Description
Replace the `ralph resume` subcommand with a `--continue` flag on the `ralph run` command to follow the idiomatic pattern used by Claude Code. This makes the CLI more intuitive: `ralph run --continue` instead of `ralph resume`.

## Background
Claude Code uses `claude --continue` to resume a previous conversation rather than a separate `resume` subcommand. This is a more natural CLI pattern because:

1. **Discoverability**: Users naturally look for flags on the main command
2. **Consistency**: Aligns with Claude Code's established patterns
3. **Simplicity**: One command (`run`) with options, not two separate commands

**Current behavior:**
```bash
ralph run -p "Start task"     # Fresh run
ralph resume                   # Resume from scratchpad
```

**Target behavior:**
```bash
ralph run -p "Start task"           # Fresh run
ralph run --continue                # Resume from scratchpad
ralph run -c                        # Short form
```

## Technical Requirements
1. Add `--continue` / `-c` flag to `RunArgs` struct
2. Remove `Commands::Resume` variant and `ResumeArgs` struct
3. Merge resume logic into `run_command()` based on `--continue` flag
4. Update error messages to reference `ralph run` instead of `ralph resume`
5. Keep backward compatibility: `ralph resume` should show deprecation warning and work
6. Update all documentation and help text
7. Update integration tests to use new flag syntax

## Dependencies
- `clap` derive macros for argument parsing
- Existing `run_loop_impl()` function (already supports resume mode)
- Integration tests in `integration_resume.rs`

## Implementation Approach

### 1. Modify RunArgs to include --continue flag
```rust
#[derive(Parser, Debug)]
struct RunArgs {
    /// Continue from existing scratchpad (resume interrupted loop)
    #[arg(long = "continue", short = 'c')]
    pub continue_mode: bool,

    // ... existing fields
}
```

### 2. Update run_command to handle continue mode
```rust
async fn run_command(..., args: RunArgs) -> Result<()> {
    let resume = args.continue_mode;

    if resume {
        // Check scratchpad exists (from current resume_command logic)
        let scratchpad_path = Path::new(&config.core.scratchpad);
        if !scratchpad_path.exists() {
            anyhow::bail!(
                "Cannot continue: scratchpad not found at '{}'. \
                 Start a fresh run with `ralph run`.",
                config.core.scratchpad
            );
        }
        info!("Found existing scratchpad, continuing from previous state");
    }

    run_loop_impl(config, color_mode, resume, ...).await
}
```

### 3. Add deprecation alias for backward compatibility
```rust
#[derive(Subcommand, Debug)]
enum Commands {
    Run(RunArgs),

    /// DEPRECATED: Use `ralph run --continue` instead
    #[command(hide = true)]
    Resume(ResumeArgs),

    // ... other commands
}
```

### 4. Update integration tests
Change all `ralph resume` calls to `ralph run --continue` in test files.

## Acceptance Criteria

1. **--continue Flag Works**
   - Given a scratchpad exists from a previous run
   - When `ralph run --continue` is executed
   - Then the loop resumes from existing scratchpad
   - And `task.resume` event is published (not `task.start`)

2. **Short Form -c Works**
   - Given a scratchpad exists
   - When `ralph run -c` is executed
   - Then the loop resumes (same as --continue)

3. **Fresh Run Still Works**
   - Given `ralph run` without --continue flag
   - When executed
   - Then a fresh run starts with `task.start` event

4. **Error Without Scratchpad**
   - Given no scratchpad exists
   - When `ralph run --continue` is executed
   - Then an error is shown: "Cannot continue: scratchpad not found..."

5. **Backward Compatibility**
   - Given `ralph resume` is called (old syntax)
   - When executed
   - Then it works but shows deprecation warning
   - And suggests using `ralph run --continue` instead

6. **Help Text Updated**
   - Given `ralph run --help`
   - When displayed
   - Then --continue flag is documented with clear description

7. **Integration Tests Pass**
   - Given updated tests using `ralph run --continue`
   - When `cargo test -p ralph-cli integration_resume` runs
   - Then all tests pass

## Files to Modify

| File | Changes |
|------|---------|
| `crates/ralph-cli/src/main.rs` | Add `--continue` to RunArgs, merge resume logic, deprecate Resume command |
| `crates/ralph-cli/tests/integration_resume.rs` | Update all `resume` calls to `run --continue` |
| `crates/ralph-cli/tests/integration_events_isolation.rs` | Update any resume-related tests |
| `CLAUDE.md` | Update any CLI examples if present |

## Metadata
- **Complexity**: Medium
- **Labels**: CLI, UX, Breaking Change, Deprecation
- **Required Skills**: Rust, Clap, CLI Design
