# Rough Idea: Claude Adapter Streaming Output for Non-Interactive Mode

## Problem Statement

When running Ralph in non-interactive mode (`ralph run -P PROMPT.md`), users cannot see Claude's output as it works through a task. Currently, when running `claude -p <prompt>`, only the final output is shown after Claude completes.

This is problematic because Ralph can be going in the wrong direction for long periods of time and the user has no visibility into what's happening.

## Desired Outcome

Extend the Claude adapter to properly parse and output from the JSON stream output format (`--output-format stream-json`), allowing users to view Claude's output in real-time as it works through a task.

## Context

The existing `claude.spec.md` documents the JSON stream output format:
- Claude emits newline-delimited JSON objects with event types: `system`, `assistant`, `user`, `result`, `stream_event`
- The spec describes how Ralph should parse these events and forward them to the TUI
- However, the current implementation may not be fully wired up for the non-interactive `ralph run` case

## Key Requirements (Initial)

1. Parse JSON stream output from Claude in real-time
2. Display assistant responses as they stream in
3. Show tool invocations and their results
4. Maintain visibility into progress during long-running tasks
5. Work correctly in non-interactive/headless mode
