## Summary

Fixes an infinite loop caused by `check_default_publishes` not recording injected topics in `seen_topics`, making `required_events` chain validation permanently unsatisfiable.

## Bug

When a hat writes no events to JSONL and the orchestrator injects a fallback via `default_publishes`, the event is published on the bus (triggering downstream hats correctly) but **never recorded in `seen_topics`**. If that topic is listed in `required_events`, `LOOP_COMPLETE` is rejected every iteration because the topic appears permanently missing — even though the event was successfully delivered and acted upon.

This creates an **unrecoverable infinite loop**: the work is done, all tests pass, but the loop can never terminate.

## Steps to Reproduce

1. Configure a preset with `required_events` that includes a topic delivered via `default_publishes`:

   ```yaml
   event_loop:
     required_events:
       - "plan.draft"
       - "plan.approved"
       - "all.built"

   hats:
     planner:
       triggers: ["research.complete"]
       publishes: ["plan.draft"]
       default_publishes: "plan.draft"
     review_gate:
       triggers: ["plan.draft"]
       publishes: ["plan.approved"]
   ```

2. Run a loop where the planner hat completes its work but does not explicitly emit `plan.draft` in its JSONL output (the agent writes the plan file but doesn't emit the event tag).

3. The orchestrator injects `plan.draft` via `check_default_publishes` — the ReviewGate triggers correctly and the workflow proceeds through all remaining phases.

4. When the final hat emits `LOOP_COMPLETE`, the event loop rejects it:

   ```
   WARN ralph_core::event_loop: Rejecting LOOP_COMPLETE: required events not seen during loop lifetime missing=["plan.draft"]
   ```

5. A `task.resume` event is injected, but no hat can retroactively produce `plan.draft`. The loop repeats indefinitely until `max_iterations` is exhausted.

## Root Cause

`check_default_publishes` (`event_loop/mod.rs:1403`) calls `self.bus.publish()` but does not call `self.state.record_topic()`. Topic recording only happens in `process_events_from_jsonl` (line 2157) for events parsed from the agent's JSONL output. The `default_publishes` code path bypasses this entirely.

```rust
// BEFORE (bug): publishes on bus but not tracked in seen_topics
pub fn check_default_publishes(&mut self, hat_id: &HatId) {
    if let Some(config) = self.registry.get_config(hat_id)
        && let Some(default_topic) = &config.default_publishes
    {
        let default_event = Event::new(default_topic.as_str(), "")
            .with_source(hat_id.clone());
        self.bus.publish(default_event);  // <-- topic never recorded
    }
}
```

## Fix

One line added to `check_default_publishes` to record the topic before publishing:

```rust
self.state.record_topic(default_topic.as_str());
self.bus.publish(default_event);
```

## Changes

| File | Change |
|------|--------|
| `crates/ralph-core/src/event_loop/mod.rs` | +1 line: `record_topic()` call in `check_default_publishes` |
| `crates/ralph-core/src/event_loop/tests.rs` | +1 assertion in existing test, +1 new regression test |

## Tests

**Updated test** (`test_default_publishes_injects_when_no_events`):
- Added assertion that `seen_topics` contains the default topic after `check_default_publishes`

**New regression test** (`test_default_publishes_satisfies_required_events_for_completion`):
- Configures `required_events: ["plan.draft", "all.built"]` with a planner hat that has `default_publishes: "plan.draft"`
- Simulates the exact failure scenario: planner writes no events, `default_publishes` injects `plan.draft`, then `all.built` arrives via JSONL
- Verifies `LOOP_COMPLETE` is accepted (previously would have been rejected indefinitely)

## Test Plan

- [x] `cargo test -p ralph-core test_default_publishes` — 4 tests pass (including new regression test)
- [x] `cargo test -p ralph-core test_chain_validation` — 5 tests pass (no regressions)
- [x] `cargo test` — full suite passes
