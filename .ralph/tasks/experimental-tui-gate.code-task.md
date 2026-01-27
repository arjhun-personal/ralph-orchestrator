---
status: obsolete
created: 2026-01-15
started: 2026-01-15
completed: 2026-01-15
obsoleted: 2026-01-19
---
# Task: Gate Interactive Mode Behind Experimental Flag

> **OBSOLETE**: This experimental gate was removed on 2026-01-19 after the TUI stabilized.
> The `--tui` flag now works without requiring any config changes.

## Description
Add an `experimental_tui` configuration flag to gate the `--tui` mode behind an explicit opt-in. When users attempt to use TUI mode without enabling this flag, emit a helpful warning and fall back to autonomous mode.

## Background
The TUI mode (`--tui` flag) is currently buggy and not ready for general use. Rather than removing the feature entirely, we want to gate it behind an experimental flag so that:
1. Users don't accidentally encounter bugs
2. Developers and testers can still access the feature for development
3. The feature remains in the codebase for continued iteration

The codebase already has a "deferred feature" pattern used for `archive_prompts` and `enable_metrics` that provides a clean model to follow.

## Technical Requirements
1. Add `experimental_tui: bool` field to `CliConfig` struct with `#[serde(default)]` (defaults to false)
2. In `run_loop_impl`, check `experimental_tui` before enabling TUI mode
3. If TUI mode is requested but `experimental_tui` is false:
   - Emit a warning explaining how to enable the feature
   - Fall back to autonomous mode (do not error out)
4. If `experimental_tui` is true, allow TUI mode to proceed normally
5. Update any relevant documentation or config examples

## Dependencies
- `crates/ralph-core/src/config.rs` - CliConfig struct definition (around line 572-606)
- `crates/ralph-cli/src/main.rs` - run_loop_impl function, mode decision logic (around line 912-923)
- Existing deferred feature pattern at `config.rs:315-327` for reference

## Implementation Approach
1. Add `experimental_tui` field to CliConfig in config.rs with serde default
2. Locate the mode decision logic in run_loop_impl (where `interactive_requested` is determined)
3. Add a check: if interactive requested AND experimental_tui is false, log warning and force autonomous
4. Use `warn!()` macro consistent with existing warning patterns
5. Run `cargo test` to ensure no regressions

## Acceptance Criteria

1. **Config Field Exists**
   - Given the CliConfig struct in config.rs
   - When a config file omits `experimental_tui`
   - Then it defaults to `false`

2. **Interactive Blocked Without Flag**
   - Given `experimental_tui` is false or unset in config
   - When user runs `ralph run --tui -c config.yml -p "test"`
   - Then a warning is emitted explaining the experimental flag and execution continues in autonomous mode

3. **Interactive Allowed With Flag**
   - Given `experimental_tui: true` in the config file
   - When user runs `ralph run --tui -c config.yml -p "test"`
   - Then interactive mode proceeds normally (assuming TTY is available)

4. **Warning Message Is Helpful**
   - Given interactive mode is blocked due to missing flag
   - When the warning is displayed
   - Then it includes the exact config field name (`cli.experimental_tui`) and how to enable it

5. **No Breaking Changes**
   - Given the changes are applied
   - When running `cargo test`
   - Then all existing tests pass

## Metadata
- **Complexity**: Low
- **Labels**: Configuration, Feature-Gate, TUI, CLI
- **Required Skills**: Rust, serde configuration, CLI argument handling
