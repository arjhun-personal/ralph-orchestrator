---
name: ralph-code-assist
description: Runs Ralph orchestrator with the code-assist configuration to implement code tasks autonomously. Spawns Ralph as a subprocess that handles the full TDD workflow (Explore, Plan, Code, Commit) for the given code task file.
type: anthropic-skill
version: "1.0"
---

# Ralph Code Assist

## Overview

This skill launches the Ralph orchestrator with the `ralph.code-assist.yml` configuration to implement code tasks autonomously. Ralph handles the full TDD workflow (Explore → Plan → Code → Commit) using its event loop and hat system.

Use this when you have a `.code-task.md` file and want Ralph to implement it end-to-end without manual intervention.

## Parameters

- **code_task** (required): Path to a `.code-task.md` file containing the task specification
- **config** (optional, default: "ralph.code-assist.yml"): Ralph configuration file to use
- **verbose** (optional, default: false): Enable verbose output from Ralph

**Constraints for parameter acquisition:**
- You MUST ask for all required parameters upfront in a single prompt
- You MUST validate that the code_task file exists before proceeding
- You MUST verify the code task file has the `.code-task.md` extension or is a valid task file

## Steps

### 1. Validate Inputs

Verify the code task file exists and is properly formatted.

**Constraints:**
- You MUST check that the code_task file path exists
- You MUST read the code task file to verify it contains valid task content
- You MUST notify the user if the file is missing or malformed
- You SHOULD display a summary of the task (name, description, complexity) before proceeding

### 2. Display Task Summary

Show the user what will be implemented before starting.

**Constraints:**
- You MUST display the task name, description, and complexity from the code task file
- You MUST list the acceptance criteria that will be implemented
- You MUST confirm the current working directory where changes will be made

### 3. Execute Ralph

Run the Ralph orchestrator with the code-assist configuration.

**Constraints:**
- You MUST run the command: `cargo run --bin ralph -- run -c {config} -p "Implement the task at: {code_task}"`
- You MUST capture and display Ralph's output in real-time or stream it to the user
- You MUST monitor for the completion promise "LOOP_COMPLETE" or error conditions
- You MUST handle timeouts gracefully (default max runtime is 8 hours)
- You SHOULD provide periodic status updates if Ralph runs for an extended time

### 4. Report Results

Summarize the execution results for the user.

**Constraints:**
- You MUST report whether Ralph completed successfully or encountered errors
- You MUST display the termination reason (completion promise, timeout, error)
- You MUST list any files created or modified during execution
- You SHOULD provide a summary of what was implemented if successful
- You SHOULD suggest next steps (review changes, run tests, commit, etc.)

## Example Usage

```
/ralph-code-assist tools/task-01-replay-backend.code-task.md
```

Or with a different config:

```
/ralph-code-assist code_task=tools/task-01-replay-backend.code-task.md config=ralph.smoke.yml
```

## Expected Output

When successful, Ralph will:
1. Read the code task specification
2. Follow the code-assist SOP (TDD workflow)
3. Implement tests first, then implementation code
4. Run tests to verify
5. Commit changes with conventional commit message
6. Emit "LOOP_COMPLETE" when done

## Troubleshooting

### Ralph Build Fails
If `cargo run --bin ralph` fails:
- You SHOULD run `cargo build --bin ralph` first to see build errors
- You SHOULD check that all dependencies are available
- You SHOULD verify the ralph.code-assist.yml config file exists

### Task Not Completing
If Ralph runs but doesn't complete:
- You SHOULD check the scratchpad at `.agent/scratchpad.md` for progress
- You SHOULD verify the task requirements are clear and achievable
- You SHOULD consider if the task needs to be broken into smaller pieces

### Agent Artifacts
Ralph creates working files during execution:
- `.agent/scratchpad.md` - Ralph's working memory between iterations
- These are normal and can be cleaned up after successful runs
- You SHOULD add `.agent/` to .gitignore if not already present

## Related

- `ralph.code-assist.yml` - The Ralph configuration for code assist mode
- `.sops/code-assist.sop.md` - The SOP that Ralph follows (if exists)
- `/code-assist` skill - Direct Claude code assist (not via Ralph)
