## Summary

Adds three engine-level mechanisms to prevent hat role violations and infinite loop cycling — Phase 2 of the loop cycling fix plan.

- **2A**: `disallowed_tools` field on `HatConfig` with prompt-level enforcement
- **2B**: Stale topic detection that terminates loops when the same event is emitted 3+ times consecutively
- **2C**: Post-iteration file-modification audit that emits `scope_violation` events

## Problem

Two bugs were observed during a ralph loop run:

**Bug 1: Dispatcher implemented code (role violation)**
The dispatcher hat read plan files and edited 12 source files despite instructions saying "Don't build anything yourself." Existing `enforce_hat_scope` only validates event publishing, not tool usage. There's no mechanism to prevent a hat from using Edit/Write/Bash tools.

**Bug 2: 5 wasted iterations cycling after work done (~$1.7 burned)**
After all work completed, the loop cycled between dispatcher and builder emitting the same events repeatedly:

```
iter 7  (builder):    nothing to do → default_publishes → build.complete
iter 8  (dispatcher): got build.complete → all.built
iter 9  (builder):    nothing to do → build.complete
iter 10 (dispatcher): got build.complete → all.built (again)
iter 11 (builder):    finally emits LOOP_COMPLETE
```

The same `all.built` → `build.complete` → `all.built` pattern was also observed in a separate run.

## Changes

### 2A: `disallowed_tools` prompt-level enforcement

| File | Change |
|------|--------|
| `crates/ralph-core/src/config.rs` | New `disallowed_tools: Vec<String>` field on `HatConfig` |
| `crates/ralph-core/src/hatless_ralph.rs` | New `disallowed_tools` on `HatInfo`; TOOL RESTRICTIONS section injected in active hat prompts |

When a hat has `disallowed_tools` configured, the prompt includes a prominent section:

```markdown
### TOOL RESTRICTIONS

You MUST NOT use these tools in this hat:
- **Edit** — blocked for this hat
- **Write** — blocked for this hat

Using a restricted tool is a scope violation.
File modifications are audited after each iteration.
```

Preset usage:
```yaml
dispatcher:
  disallowed_tools: ['Edit', 'Write', 'NotebookEdit']
```

### 2B: Stale topic detection (cycle breaker)

| File | Change |
|------|--------|
| `crates/ralph-core/src/event_loop/loop_state.rs` | New `last_emitted_topic` and `consecutive_same_topic` fields; `record_topic()` tracks consecutive emissions |
| `crates/ralph-core/src/event_loop/mod.rs` | New `TerminationReason::LoopStale` variant; `check_termination()` returns `LoopStale` when same topic emitted 3+ times |

When the same topic is emitted 3 or more times consecutively, the loop terminates with exit code 1 (`LoopStale`). This catches the `all.built` → `build.complete` → `all.built` cycle pattern.

The tracking is done in `record_topic()` which is called both from `process_events_from_jsonl()` (agent-written events) and `check_default_publishes()` (auto-injected events), ensuring full coverage.

### 2C: File-modification audit (hard enforcement)

| File | Change |
|------|--------|
| `crates/ralph-core/src/event_loop/mod.rs` | New `audit_file_modifications()` method called from `process_output()` |

After each iteration, if the active hat has `Edit` or `Write` in `disallowed_tools`, runs `git diff --stat HEAD` to detect unauthorized file modifications. If modifications are found, emits a `<hat_id>.scope_violation` event on the bus.

Presets can route this event to trigger corrective actions:
```yaml
final_committer:
  triggers: ['all.built', 'dispatcher.scope_violation']
```

### Exhaustive match updates

| File | Change |
|------|--------|
| `crates/ralph-cli/src/display.rs` | Added `LoopStale` to termination display |
| `crates/ralph-cli/src/loop_runner.rs` | Added `LoopStale` to history recording and merge queue state |
| `crates/ralph-core/src/summary_writer.rs` | Added `LoopStale` to summary status text |
| `crates/ralph-bench/src/main.rs` | Added `LoopStale` to benchmark result formatting |

## Tests

- [x] `cargo test` — full workspace passes (all existing tests + new field defaults)
- [x] `cargo build` — clean compilation

## Test Plan

- [ ] Configure a preset with `disallowed_tools: ['Edit', 'Write']` on dispatcher → verify TOOL RESTRICTIONS section appears in prompt
- [ ] Run a loop where the same event cycles 3+ times → verify `LoopStale` termination
- [ ] Run a loop where a restricted hat modifies files → verify `scope_violation` event emitted
