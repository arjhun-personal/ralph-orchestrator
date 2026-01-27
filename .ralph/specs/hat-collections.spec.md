---
status: draft
gap_analysis: 2026-01-14  # Updated: preflight_check removed, Hatless Ralph is universal fallback
related:
  - event-loop.spec.md
---

# Hat Collections Specification

## Overview

Hat collections are pre-configured sets of hats (agent personas) designed for specific workflows. This spec defines the contract for valid hat collections.

**Important: Hatless Ralph Architecture**

With the Hatless Ralph redesign, many traditional validation concerns are now handled automatically:

- **Empty hats are valid** — Ralph runs in "solo mode" with zero hats, doing all work himself
- **Orphan events have a fallback** — Ralph catches all unhandled events as the universal fallback
- **No entry point is fine** — Ralph handles `task.start` and `task.resume` if no hat subscribes
- **No dead ends** — Orphaned events fall through to Ralph, who can handle or complete

The only remaining hard validation is **ambiguous routing** (two hats claiming the same trigger).

## Goals

1. **Fail fast on ambiguous routing** — Two hats cannot both claim the same event trigger
2. **Clear errors** — Rejections include what's wrong and how to fix it
3. **Flexibility** — Users can configure zero hats (solo mode) or many hats (team mode)

## Hat Collection Contract

A valid hat collection MUST satisfy:

### Required Properties

| Property | Requirement |
|----------|-------------|
| **Unique triggers** | Each trigger pattern maps to exactly one hat (no ambiguous routing) |

### Properties Handled by Ralph (No Longer Validated)

| Property | Why Not Validated |
|----------|-------------------|
| At least one hat | Ralph runs solo when `hats: {}` — valid configuration |
| Entry point exists | Ralph is the universal fallback for `task.start` |
| Exit point exists | Ralph owns `LOOP_COMPLETE`, always available |
| Reachable hats | Unreachable hats are wasteful but not fatal |
| No dead ends | Orphaned events fall to Ralph as universal fallback |

### Hat Definition Schema

Each hat in a collection must conform to:

```yaml
<hat_id>:
  name: string              # Human-readable name (required)
  triggers: [string]        # Events that activate this hat (required, non-empty)
  publishes: [string]       # Events this hat can emit (optional, defaults to [])
  instructions: string      # Custom instructions for this hat (optional)
```

**Note on self-routing:** A hat MAY trigger on events it also publishes (self-routing). This is allowed and is NOT considered "ambiguous routing." Ambiguous routing only occurs when two DIFFERENT hats trigger on the same event. See `event-loop.spec.md` section "Self-Routing Is Allowed" for rationale.

### Terminal Events (Historical Context)

> **Note:** With Hatless Ralph, terminal events are no longer needed for validation since Ralph catches all orphan events. This section is preserved for historical context only.

The `completion_promise` (default: `LOOP_COMPLETE`) is the only meaningful terminal event — it signals Ralph to exit the loop.

### Recovery Mechanism

> **Hatless Ralph change:** There is no longer a "recovery hat" requirement. Ralph IS the universal recovery mechanism.

When events have no hat subscriber (including `task.resume` and blocked events), Ralph catches them as the universal fallback and decides how to proceed.

## Validation Rules

### Rule 1: Unique Triggers (Ambiguous Routing)

This is the **only hard validation** remaining after Hatless Ralph:

```
ERROR: Ambiguous routing for trigger 'build.done'.

Both 'planner' and 'reviewer' trigger on 'build.done'. Ralph cannot
determine which hat should handle the event.

Fix: Ensure each trigger maps to exactly one hat:

  planner:
    triggers: ["task.start", "review.done"]  # Remove build.done
  reviewer:
    triggers: ["build.done"]                 # Unique ownership
```

### Rules Removed by Hatless Ralph

The following rules were part of the original design but are **no longer enforced** because Ralph acts as a universal fallback:

| Former Rule | Why Removed |
|-------------|-------------|
| Non-empty collection | Ralph runs solo when `hats: {}` |
| Entry point exists | Ralph handles `task.start` if no hat subscribes |
| No orphan events | Orphaned events fall through to Ralph |
| Reachable hats | Unreachable hats are wasteful but not fatal |
| Recovery hat valid | Ralph IS the recovery mechanism |
| Exit point exists | Ralph owns `LOOP_COMPLETE` |

## Event Flow Graph (Optional Analysis)

Event flow graph analysis is **no longer required for validation** but may be useful for debugging or visualization:

```
                     analyzes
┌──────────────┐  ────────────▶  ┌─────────────────────────┐
│ Hat Collection│                │   Event Flow Graph      │
└──────────────┘                 └─────────────────────────┘
                                          │
                                          ▼
                                ┌─────────────────┐
                                │ Trigger Conflict│
                                │    Detection    │
                                └─────────────────┘
```

### Simplified Algorithm

With Hatless Ralph, the algorithm is reduced to:

1. **Build trigger map**: For each event, identify which hat(s) trigger on it
2. **Check for conflicts**: If more than one hat triggers on the same event, reject

## Configuration Examples

### Valid: Minimal Collection (Single Hat)

```yaml
hats:
  worker:
    name: "Worker"
    triggers: ["task.start", "task.resume"]
    # No publishes - single hat runs to completion
    instructions: |
      Implement the requested feature. When done, output: LOOP_COMPLETE
```

**Why valid:**
- Has entry point (`task.start`)
- Single hat is implicitly the recovery hat
- Single hat can emit completion promise

### Valid: Standard Planner/Builder

```yaml
hats:
  planner:
    name: "Planner"
    triggers: ["task.start", "task.resume", "build.done", "build.blocked"]
    publishes: ["build.task"]

  builder:
    name: "Builder"
    triggers: ["build.task"]
    publishes: ["build.done", "build.blocked"]
```

**Why valid:**
- Entry: `planner` triggers on `task.start`
- Recovery: `planner` triggers on `task.resume` and `build.blocked`
- No dead ends: `build.task` → `builder`, `build.done/blocked` → `planner`
- Both hats reachable: `task.start` → `planner` → `build.task` → `builder`

### Valid: Extended Team with Reviewer

```yaml
hats:
  planner:
    name: "Planner"
    triggers: ["task.start", "task.resume", "build.blocked", "review.approved", "review.rejected"]
    publishes: ["build.task", "review.request"]

  builder:
    name: "Builder"
    triggers: ["build.task"]
    publishes: ["build.done", "build.blocked"]

  reviewer:
    name: "Reviewer"
    triggers: ["build.done", "review.request"]
    publishes: ["review.approved", "review.rejected"]

event_loop:
  completion_promise: "LOOP_COMPLETE"
```

**Why valid:**
- Entry: `planner` → `task.start`
- All hats reachable via event flow
- All published events have subscribers
- `planner` handles blocked events (recovery)

### Valid Under Hatless Ralph: Orphan Event

```yaml
hats:
  planner:
    name: "Planner"
    triggers: ["task.start"]
    publishes: ["build.task", "deploy.start"]  # ← deploy.start has no subscriber

  builder:
    name: "Builder"
    triggers: ["build.task"]
    publishes: ["build.done"]
```

**Why valid:** When `planner` publishes `deploy.start`, Ralph (as the universal fallback) catches it and handles the orphaned event.

### Valid Under Hatless Ralph: Unreachable Hat

```yaml
hats:
  planner:
    name: "Planner"
    triggers: ["task.start", "build.done"]
    publishes: ["build.task"]

  builder:
    name: "Builder"
    triggers: ["build.task"]
    publishes: ["build.done"]

  auditor:
    name: "Auditor"
    triggers: ["audit.request"]  # ← No hat publishes audit.request
    publishes: ["audit.done"]
```

**Why valid:** Unreachable hats are wasteful but not fatal. The `auditor` simply never executes.

### Invalid: Ambiguous Routing

```yaml
hats:
  planner:
    name: "Planner"
    triggers: ["task.start", "build.done"]  # ← build.done also in reviewer
    publishes: ["build.task"]

  builder:
    name: "Builder"
    triggers: ["build.task"]
    publishes: ["build.done"]

  reviewer:
    name: "Reviewer"
    triggers: ["build.done"]  # ← Conflicts with planner
    publishes: ["review.done"]
```

**Error:**
```
ERROR: Ambiguous routing for trigger 'build.done'.
Both 'planner' and 'reviewer' trigger on 'build.done'.
```

### Valid Under Hatless Ralph: No Recovery Path

```yaml
hats:
  coordinator:
    name: "Coordinator"
    triggers: ["task.start", "impl.done"]  # ← Missing task.resume
    publishes: ["impl.task"]

  implementer:
    name: "Implementer"
    triggers: ["impl.task"]
    publishes: ["impl.done", "impl.blocked"]  # ← blocked has no handler
```

**Why valid:** Ralph IS the recovery mechanism. When `task.resume` is published, Ralph catches it. When `impl.blocked` is published, Ralph catches it as the universal fallback.

## Preset Collections

Ralph ships with preset collections in the `presets/` directory:

| Preset | Purpose | Hats | Terminal Events |
|--------|---------|------|-----------------|
| `feature.yml` | Feature development | planner, builder, reviewer | — |
| `feature-minimal.yml` | Feature dev (auto-derived instructions) | planner, builder, reviewer | — |
| `research.yml` | Code exploration (no changes) | researcher, synthesizer | `research.question`, `synthesis.complete` |
| `docs.yml` | Documentation writing | planner, writer, reviewer | — |
| `refactor.yml` | Safe code refactoring | planner, refactorer, verifier | — |
| `debug.yml` | Bug investigation | investigator, tester, fixer, verifier | `hypothesis.confirmed`, `fix.blocked`, `fix.failed` |
| `review.yml` | Code review | reviewer, analyzer | `review.complete` |
| `deploy.yml` | Deployment workflow | planner, builder, deployer, verifier | — |
| `gap-analysis.yml` | Spec vs implementation comparison | analyzer, verifier, reporter | — |

**Note:** Some presets intentionally have "orphan" events that represent workflow completion or hand-off points. These should be declared as `terminal_events` once that feature is implemented. Until then, these presets rely on the completion promise mechanism.

## Acceptance Criteria

### Validation Errors (Hatless Ralph Model)

- **Given** a hat collection where two hats trigger on the same event
- **When** configuration is loaded
- **Then** error "Ambiguous routing for trigger" is returned with both hat names

### Valid Collections (Hatless Ralph Model)

- **Given** a hat collection with no hats (`hats: {}`)
- **When** configuration is loaded
- **Then** validation passes (Ralph runs in solo mode)

- **Given** a hat collection where no hat triggers on `task.start`
- **When** configuration is loaded
- **Then** validation passes (Ralph handles `task.start` as universal fallback)

- **Given** a hat publishes an event with no subscriber
- **When** configuration is loaded
- **Then** validation passes (orphaned events fall through to Ralph)

- **Given** a single-hat collection that triggers on any event
- **When** configuration is loaded
- **Then** validation passes

- **Given** a standard planner/builder collection
- **When** configuration is loaded
- **Then** validation passes with no warnings

### Error Messages

- **Given** an ambiguous routing error
- **When** error is displayed
- **Then** it includes: the conflicting trigger, both hat names, and how to fix it

### Preset Validation

- **Given** any preset in `presets/` directory
- **When** loaded with default settings
- **Then** validation passes with no errors or warnings

## Implementation Notes

### Crate Placement

| Component | Crate |
|-----------|-------|
| `HatCollectionValidator` | `ralph-core` |
| `EventFlowGraph` | `ralph-core` |
| Validation error types | `ralph-core` |
| Preset loading | `ralph-cli` |

### Validation Timing

Validation runs at config load time, before any iteration starts. This ensures:
1. Fast feedback (no wasted iterations)
2. Clear separation between config errors and runtime errors
3. Ability to use `ralph validate` command for CI/CD checks

### Migration Path

Existing configurations that would fail new validation rules:
1. First release: New validations emit warnings only (with deprecation notice)
2. Second release: New validations are errors by default, `strict_validation: false` available

## Implementation Status

This section tracks what's currently implemented.

### Currently Implemented (in `config.rs`)

| Validation | Status | Location |
|------------|--------|----------|
| Unique triggers (no ambiguous routing) | ✅ Implemented | `validate()` |

### Removed (Hatless Ralph Makes These Obsolete)

The following validations were previously in `preflight_check()` but have been **removed** because Hatless Ralph provides universal fallback:

| Validation | Why Removed |
|------------|-------------|
| Non-empty collection | Ralph runs in "solo mode" when `hats: {}` |
| Entry point exists | Ralph is the universal fallback for unhandled events |
| No orphan events | Orphaned events fall through to Ralph |
| Reachability check | Unreachable hats are wasteful, not fatal |
| Recovery hat validation | Ralph IS the recovery mechanism |

### Optional Future Enhancements

These are no longer blockers but could improve developer experience:

| Feature | Priority | Notes |
|---------|----------|-------|
| Rich error messages with fix suggestions | Low | Improves DX for ambiguous routing |
| Warning for unreachable hats | Low | Help users identify dead code |
