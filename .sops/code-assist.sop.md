# Code Assist

## Overview

This SOP guides implementation of code tasks using test-driven development, following an Explore, Plan, Code, Commit workflow. It balances automation with user collaboration while adhering to existing patterns and prioritizing readability and extensibility.

## Parameters

- **task_description** (required): Task to be implemented
- **additional_context** (optional): Supplementary information
- **documentation_dir** (optional, default: ".sop/planning"): Directory for planning documentation
- **repo_root** (optional, default: current working directory): Repository root path
- **task_name** (optional): Short descriptive name for the task
- **mode** (optional, default: "auto"): Execution mode - "interactive" or "auto"

**Constraints:**
- You MUST ask for all parameters upfront in a single prompt, not just required ones because this ensures efficient workflow and prevents repeated interruptions during execution

## Mode Behavior

### Interactive Mode
- Present proposed actions for confirmation
- Explain pros/cons when multiple approaches exist
- Review artifacts for feedback
- Ask clarifying questions about ambiguous requirements

### Auto Mode
- Execute autonomously
- Document decisions in progress.md
- Select most appropriate approaches when alternatives exist
- Provide comprehensive summaries at completion

## Critical Separation of Concerns

Documentation about implementation goes in the documentation directory; actual code (tests and implementations) must be in the repository root. Never place code files in documentation directories.

## Steps

### 1. Setup

Initialize documentation and discover project context.

**Constraints:**
- You MUST validate and create documentation directory structure
- You MUST discover instruction files, particularly CODEASSIST.md
- You MUST create context.md and progress.md files
- You MUST document project structure, requirements, patterns, and dependencies

### 2. Explore Phase

Analyze requirements and research existing patterns.

**Constraints:**
- You MUST analyze requirements and identify functional criteria
- You MUST research existing patterns and create dependency maps
- You MUST compile findings into comprehensive context documentation
- You MUST focus on high-level concepts rather than implementation code

### 3. Plan Phase

Design test strategy and implementation approach.

**Constraints:**
- You MUST design test strategy covering normal operation and edge cases
- You MUST create implementation plan with high-level structure
- You MUST save test scenarios and implementation planning to plan.md
- You MUST use diagrams or pseudocode rather than actual implementation code

### 4. Code Phase

Implement using TDD principles.

**Constraints:**
- You MUST implement test cases following TDD principles
- You MUST develop implementation using RED → GREEN → REFACTOR cycle
- You MUST refactor to align with existing code conventions
- You MUST validate that all tests pass and builds succeed

### 5. Commit Phase

Create conventional commit with all changes.

**Constraints:**
- You MUST follow Conventional Commits specification
- You MUST stage all relevant files using git add
- You MUST execute git commit with prepared message
- You MUST document commit hash and status
- You MUST NOT push to remote repositories

## Desired Outcome

A complete, well-tested implementation meeting specifications with clean, documented code following existing patterns while prioritizing readability and extensibility.

## Key Constraints Summary

- All parameters must be acquired upfront
- Tests must be implemented before any implementation code
- No code implementations in documentation directories
- Tests must fail initially before implementation
- Build and test verification required before commits
- No pushing to remote repositories
- Documentation uses markdown checklists for progress tracking
