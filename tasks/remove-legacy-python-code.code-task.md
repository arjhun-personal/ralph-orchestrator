---
status: completed
created: 2026-01-14
started: 2026-01-14
completed: 2026-01-14
---
# Task: Remove Legacy v1 Python Code

## Description
Remove all legacy Python code from the ralph-orchestrator repository. The project has been fully migrated from Python (v1) to Rust (v2.0), and the Python code now exists only as a deprecated "tombstone" package. This cleanup will reduce maintenance burden and eliminate confusion.

## Background
The ralph-orchestrator was originally written in Python and has been completely rewritten in Rust for v2.0. The Python codebase currently serves as a deprecated placeholder that redirects users to install the Rust version via Homebrew or Cargo. With the Rust version stable and in use, the Python code is no longer needed and should be removed.

**Current Python footprint:**
- Source: `src/ralph_orchestrator/` (2 files, ~73 lines) - tombstone package
- Tests: `tests/` directory (40+ files, ~16,094 lines) - legacy test suite
- Examples: `examples/` (2 files, ~312 lines) - Python usage examples
- Benchmark: `bench/tasks/fizzbuzz-tdd/test_fizzbuzz.py` (~50 lines)
- Config: `pyproject.toml` - marked as DEPRECATED/Inactive
- Virtual env: `.venv/` - Python virtual environment

## Reference Documentation
**Required:**
- `pyproject.toml` - Shows deprecated status and package structure
- `AGENTS.md` / `CLAUDE.md` - Project guidelines (confirms Rust is primary)

## Technical Requirements
1. Remove the Python source package directory: `src/ralph_orchestrator/`
2. Remove the Python test suite: `tests/` directory
3. Remove Python example files: `examples/` directory
4. Remove Python benchmark file: `bench/tasks/fizzbuzz-tdd/test_fizzbuzz.py`
5. Remove Python configuration: `pyproject.toml`
6. Remove Python virtual environment: `.venv/` directory
7. Remove any Python-related CI/CD configuration if present
8. Remove any Python-related entries from `.gitignore` (optional, low priority)
9. Verify the Rust build still works after removal

## Dependencies
- Git for tracking removals
- Cargo for verifying Rust build still works
- No Python dependencies needed (we're removing, not modifying)

## Implementation Approach
1. Identify all Python-related files and directories
2. Remove directories in order: `.venv/`, `src/`, `tests/`, `examples/`
3. Remove individual files: `pyproject.toml`, benchmark Python file
4. Check for any remaining `.py` files or Python references
5. Run `cargo build` and `cargo test` to verify Rust codebase unaffected
6. Stage and prepare changes for commit (do not commit unless asked)

## Acceptance Criteria

1. **Python Source Removed**
   - Given the `src/ralph_orchestrator/` directory exists
   - When the cleanup is complete
   - Then the directory and all its contents are deleted

2. **Python Tests Removed**
   - Given the `tests/` directory with Python test files exists
   - When the cleanup is complete
   - Then the entire `tests/` directory is deleted

3. **Python Examples Removed**
   - Given the `examples/` directory with Python examples exists
   - When the cleanup is complete
   - Then the entire `examples/` directory is deleted

4. **Python Config Removed**
   - Given `pyproject.toml` exists in the repository root
   - When the cleanup is complete
   - Then `pyproject.toml` is deleted

5. **Virtual Environment Removed**
   - Given `.venv/` directory exists
   - When the cleanup is complete
   - Then the `.venv/` directory is deleted

6. **Benchmark Python File Removed**
   - Given `bench/tasks/fizzbuzz-tdd/test_fizzbuzz.py` exists
   - When the cleanup is complete
   - Then the file is deleted

7. **Rust Build Unaffected**
   - Given all Python code has been removed
   - When running `cargo build`
   - Then the build succeeds without errors

8. **Rust Tests Pass**
   - Given all Python code has been removed
   - When running `cargo test`
   - Then all Rust tests pass

9. **No Orphaned Python Files**
   - Given the cleanup is complete
   - When searching for `*.py` files in the repository
   - Then no Python files remain (excluding any in `.git/`)

## Metadata
- **Complexity**: Low
- **Labels**: Cleanup, Migration, Python, Legacy Removal
- **Required Skills**: File system operations, Git, basic Cargo commands
