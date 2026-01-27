---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Update CLI Arguments

## Description
Remove `-i`/`--interactive` flags and make `--tui` the primary flag for enabling TUI observation mode. This is a clean break with no backward compatibility (per project policy in CLAUDE.md:184).

## Background
Currently the TUI is enabled via `-i`/`--interactive` flags with `--tui` as a hidden deprecated alias. This task inverts that: `--tui` becomes primary with no aliases.

## Reference Documentation
**Required:**
- Design: specs/tui-observation-mode/design.md (Section 4.1)

**Additional References:**
- specs/tui-observation-mode/context.md (codebase patterns)
- specs/tui-observation-mode/plan.md (overall strategy)

**Note:** You MUST read the design document before beginning implementation.

## Technical Requirements
1. Remove `interactive: bool` field from `RunArgs` struct
2. Remove hidden `tui: bool` field (deprecated alias) from `RunArgs`
3. Add new `tui: bool` field with `#[arg(long, conflicts_with = "autonomous")]`
4. Update any code that checks `args.interactive || args.tui` to just `args.tui`
5. Apply same changes to `ResumeArgs` struct

## Dependencies
- None (first task in sequence)

## Implementation Approach
1. Read `crates/ralph-cli/src/main.rs` to understand current flag structure
2. Modify `RunArgs` struct: remove interactive, update tui
3. Modify `ResumeArgs` struct: same changes
4. Update any usages of these flags in the run/resume logic
5. Verify with `cargo build` and `ralph run --help`

## Acceptance Criteria

1. **TUI flag is primary**
   - Given the CLI is built
   - When running `ralph run --help`
   - Then `--tui` appears as a visible option with no `-i` short flag

2. **Interactive flag removed**
   - Given the CLI is built
   - When running `ralph run -i`
   - Then error message shows "Found argument '-i' which wasn't expected"

3. **Interactive long flag removed**
   - Given the CLI is built
   - When running `ralph run --interactive`
   - Then error message shows "Found argument '--interactive' which wasn't expected"

4. **TUI mode works**
   - Given a valid config file exists
   - When running `ralph run --tui -c config.yml -p "test"`
   - Then TUI mode launches (alternate screen, header visible)

5. **Build succeeds**
   - Given all changes are made
   - When running `cargo build`
   - Then compilation succeeds with no errors

## Metadata
- **Complexity**: Low
- **Labels**: cli, breaking-change
- **Required Skills**: Rust, clap argument parsing
