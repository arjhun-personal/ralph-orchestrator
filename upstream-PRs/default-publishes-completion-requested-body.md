## Summary

Fixes an infinite loop caused by `check_default_publishes` not setting `completion_requested` when the injected topic matches the `completion_promise`. The loop spins forever — the completion event exists on the bus but `check_completion_event()` never fires because the flag is only set by `process_events_from_jsonl()`, which `default_publishes` bypasses entirely.

## Bug

There are two independent code paths that can emit the `completion_promise` event:

1. **JSONL path** (`process_events_from_jsonl`): Agent writes `LOOP_COMPLETE` to events JSONL → parsed → `completion_requested = true` → `check_completion_event()` fires → loop terminates.
2. **default_publishes path** (`check_default_publishes`): Agent writes no events → orchestrator injects default event → published to bus → but `completion_requested` is **never set** → `check_completion_event()` returns `None` → loop continues forever.

The result: the final hat's `default_publishes: "LOOP_COMPLETE"` fires a `LOOP_COMPLETE` event on the bus (which wakes downstream/wildcard hats), but the loop never terminates. It cycles endlessly between hats that keep re-activating each other.

### Observed behavior

In a lexis-feature preset with 8 hats:

```
Iteration 8:  dispatcher → publishes dispatch.start
Iteration 9:  builder → completes, writes no events
              → default_publishes injects "all.built"
              → check_completion_event: completion_requested=false → continues
Iteration 10: dispatcher → re-triggered by all.built
Iteration 11: builder → same cycle repeats
...forever
```

## Steps to Reproduce

1. Configure a preset where the final hat has `default_publishes` matching `completion_promise`:

   ```yaml
   event_loop:
     completion_promise: "LOOP_COMPLETE"
     required_events:
       - "all.built"

   hats:
     final_committer:
       triggers: ["all.built"]
       publishes: ["LOOP_COMPLETE"]
       default_publishes: "LOOP_COMPLETE"
       instructions: "Verify all work is complete and emit LOOP_COMPLETE"
   ```

2. Run the loop. The final_committer hat activates when `all.built` arrives.

3. The agent completes its work but does not explicitly write `LOOP_COMPLETE` to JSONL (this is common — agents follow hat instructions imperfectly).

4. `check_default_publishes` injects `LOOP_COMPLETE` on the bus.

5. `check_completion_event()` checks `completion_requested` → `false` → returns `None`.

6. The `LOOP_COMPLETE` event on the bus wakes the wildcard subscriber (ralph), which re-dispatches to the next triggered hat, starting a new cycle.

7. The loop never terminates.

## Root Cause

`check_default_publishes` (`event_loop/mod.rs:1403`) calls `self.bus.publish()` but never sets `self.state.completion_requested`. This flag is only set in `process_events_from_jsonl` (line ~2157) when parsing events from the agent's JSONL output. The `default_publishes` path bypasses JSONL entirely, so the flag is never set.

```rust
// BEFORE (bug): publishes on bus but completion_requested stays false
pub fn check_default_publishes(&mut self, hat_id: &HatId) {
    if let Some(config) = self.registry.get_config(hat_id)
        && let Some(default_topic) = &config.default_publishes
    {
        let default_event = Event::new(default_topic.as_str(), "")
            .with_source(hat_id.clone());
        self.state.record_topic(default_topic.as_str());
        self.bus.publish(default_event);
        // <-- completion_requested never set, even if topic == completion_promise
    }
}
```

## Fix

Added a check: if the default topic matches `completion_promise`, set `completion_requested = true` directly:

```rust
self.state.record_topic(default_topic.as_str());

// If the default topic is the completion promise, set the flag directly.
// The normal path (process_events_from_jsonl) sets this when reading from
// JSONL, but default_publishes bypasses JSONL entirely.
if default_topic.as_str() == self.config.event_loop.completion_promise {
    info!(
        hat = %hat_id.as_str(),
        topic = %default_topic,
        "default_publishes matches completion_promise — requesting termination"
    );
    self.state.completion_requested = true;
}

self.bus.publish(default_event);
```

## Changes

| File | Change |
|------|--------|
| `crates/ralph-core/src/event_loop/mod.rs` | +13 lines: completion_requested check in `check_default_publishes`, doc comment update |
| `crates/ralph-core/src/event_loop/tests.rs` | +55 lines: new regression test |

## Tests

**New regression test** (`test_default_publishes_completion_promise_triggers_termination`):
- Configures `completion_promise: "LOOP_COMPLETE"` with `required_events: ["all.built"]`
- Creates a `final_committer` hat with `default_publishes: "LOOP_COMPLETE"`
- Satisfies `required_events` by writing `all.built` via JSONL
- Calls `check_default_publishes` (simulating agent writing no events)
- Asserts `check_completion_event()` returns `Some(TerminationReason::CompletionPromise)`
- Previously this returned `None`, causing the infinite loop

## Test Plan

- [x] `cargo test -p ralph-core test_default_publishes` — 5 tests pass (including new regression test)
- [x] `cargo test -p ralph-core` — 703 tests pass (no regressions)
- [x] `cargo test` — full workspace passes
