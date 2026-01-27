---
status: completed
created: 2026-01-15
started: 2026-01-15
completed: 2026-01-15
---
# Task: Implement `ralph clean` Command

## Description
Add a new `ralph clean` subcommand to the CLI that removes Ralph-generated artifacts in the `.agent/` directory. This command helps users clean up orchestration state between runs or when troubleshooting issues.

## Background
Ralph creates several artifacts during execution that are stored in the `.agent/` directory:
- `scratchpad.md` - Shared state between hats
- `events.jsonl` - Event history log
- `summary.md` - Loop summary on termination

Users need a convenient way to clean up these artifacts, especially when:
- Starting fresh on a new task
- Troubleshooting issues with stale state
- Preparing for a clean test run
- Resetting after a failed or interrupted loop

## Technical Requirements

1. Add a new `Clean` subcommand to the CLI (`ralph clean`)
2. Support optional `-c/--config` parameter to specify config file (consistent with other subcommands)
3. Read the `.agent/` directory path from config (`core.scratchpad` parent directory)
4. Delete the entire `.agent/` directory and its contents
5. Support `--dry-run` flag to preview what would be deleted without actually deleting
6. Provide clear, color-coded user feedback about what was cleaned
7. Handle edge cases gracefully:
   - Directory doesn't exist (not an error, just inform user)
   - Permission errors (display helpful error message)
   - Partial cleanup failures (report which files failed)
8. Include unit tests for the cleanup logic

## Dependencies
- Existing CLI framework (`clap` for argument parsing)
- Existing config loading infrastructure (`RalphConfig::from_file`)
- Existing color mode infrastructure (`ColorMode`)
- Standard library `fs` module for file operations

## Implementation Approach

1. **Add Clean subcommand to CLI enum**
   - Add `Clean(CleanArgs)` variant to `Commands` enum in `main.rs`
   - Define `CleanArgs` struct with `--dry-run` flag

2. **Implement cleanup logic**
   - Create `clean_command()` function that:
     - Loads config to determine `.agent/` directory path
     - Extracts parent directory from `core.scratchpad` path
     - Checks if directory exists
     - In dry-run mode: lists files that would be deleted
     - In normal mode: deletes the directory recursively
     - Reports results with appropriate color coding

3. **Add user feedback**
   - Use color-coded output consistent with other commands
   - Show what files/directories are being cleaned
   - Provide confirmation message on success
   - Display helpful error messages on failure

4. **Write tests**
   - Test cleanup with existing `.agent/` directory
   - Test cleanup when directory doesn't exist
   - Test dry-run mode
   - Test error handling for permission issues

## Acceptance Criteria

1. **Basic Cleanup**
   - Given a Ralph project with `.agent/` directory containing artifacts
   - When user runs `ralph clean`
   - Then the `.agent/` directory and all its contents are deleted
   - And user receives confirmation message

2. **Config Path Support**
   - Given a custom config file path
   - When user runs `ralph clean -c custom.yml`
   - Then the cleanup uses the `core.scratchpad` path from that config
   - And the appropriate `.agent/` directory is cleaned

3. **Dry Run Mode**
   - Given a Ralph project with `.agent/` directory
   - When user runs `ralph clean --dry-run`
   - Then files that would be deleted are listed
   - And no actual deletion occurs
   - And the `.agent/` directory still exists after the command

4. **Graceful Handling of Missing Directory**
   - Given a Ralph project without `.agent/` directory
   - When user runs `ralph clean`
   - Then the command succeeds with informational message
   - And exit code is 0 (success)

5. **Color Output**
   - Given the `--color` flag is set
   - When user runs `ralph clean --color always`
   - Then output uses ANSI color codes (green for success, yellow for warnings)
   - When `--color never` is used
   - Then output contains no ANSI color codes

6. **Error Handling**
   - Given a `.agent/` directory with permission issues
   - When user runs `ralph clean`
   - Then a clear error message is displayed
   - And exit code is non-zero

7. **Unit Test Coverage**
   - Given the cleanup implementation
   - When running the test suite
   - Then all cleanup scenarios have corresponding unit tests with >80% coverage
   - And edge cases (missing dir, permissions) are tested

## Metadata
- **Complexity**: Low
- **Labels**: CLI, Cleanup, User Experience, File Operations
- **Required Skills**: Rust, CLI development, file system operations, testing
