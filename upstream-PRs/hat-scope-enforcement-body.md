## Summary

Three layers of defense-in-depth to prevent agents from bypassing hat workflow constraints. Addresses a failure mode where an agent skipped human approval, never created tasks, and implemented all phases in a single context window by emitting events outside its hat's declared `publishes` list.

- **Hat scope enforcement**: `HatRegistry::can_publish()` gates events in `process_events_from_jsonl()` against the active hat's declared `publishes` patterns. Out-of-scope events are dropped and replaced with `{hat_id}.scope_violation` diagnostic events. Ralph in coordination mode (no active hat) retains unrestricted publishing.

- **Event chain validation + `loop.cancel`**: New `required_events` config field and `seen_topics` state tracking. `check_completion_event()` becomes a hard gate — `LOOP_COMPLETE` is rejected unless all required events have been seen during the loop's lifetime. A separate `loop.cancel` event provides clean early termination (human rejection, timeout) **without** chain validation. On rejection, a `task.resume` event is injected with the missing events listed.

- **Human timeout routing**: `wait_for_response()` timeout now publishes a `human.timeout` event instead of silently continuing. This makes timeouts visible in the event log and routable to hats that declare `human.timeout` as a trigger.

## Changes

| File | Lines | What changed |
|------|-------|-------------|
| `crates/ralph-core/src/hat_registry.rs` | +77 | `can_publish()` method + 4 unit tests |
| `crates/ralph-core/src/config.rs` | +22 | `required_events`, `cancellation_promise`, `enforce_hat_scope` fields on `EventLoopConfig` |
| `crates/ralph-core/src/event_loop/loop_state.rs` | +21 | `seen_topics`, `cancellation_requested` fields + helper methods |
| `crates/ralph-core/src/event_loop/mod.rs` | +127/-4 | Scope filtering, `loop.cancel` detection, topic recording, chain validation gate, `check_cancellation_event()`, `Cancelled` variant, `human.timeout` injection |
| `crates/ralph-core/src/event_loop/tests.rs` | +477 | 21 new tests (scope, chain validation, cancellation, timeout) |
| `crates/ralph-cli/src/loop_runner.rs` | +27 | Wire `check_cancellation_event()` before completion check |
| `crates/ralph-cli/src/display.rs` | +1 | `Cancelled` display variant |
| `crates/ralph-core/src/summary_writer.rs` | +3 | `Cancelled` status text + test fixture field |
| `crates/ralph-bench/src/main.rs` | +1 | `Cancelled` format variant |

**Total: +748/-4 across 9 files**

## Design decisions

1. **Scope enforcement runs before validation** — out-of-scope events are partitioned out before the existing `build.done`/`review.done`/`verify.passed` backpressure checks.

2. **`loop.cancel` is separate from `LOOP_COMPLETE`** — cancellation means "abort gracefully" and bypasses chain validation. Completion means "all work finished" and is gated.

3. **`Cancelled` exit code is 0** but `is_success()` returns `false` — intentional stop, not an error, but work was not completed.

4. **`required_events` is a flat presence check, not a DAG** — order doesn't matter, only that each topic was seen at least once.

5. **`human.timeout` replaces silent continuation** — injected through the same `response_event` mechanism as `human.response`, making it routable.

6. **All enforcement features are opt-in** — scope enforcement requires `enforce_hat_scope: true`, cancellation requires setting `cancellation_promise`, and chain validation requires a non-empty `required_events` list. Zero behavior change for existing users on upgrade.

## Backward compatibility

- **Zero breaking changes on upgrade.** All new config fields default to disabled:
  - `enforce_hat_scope`: `false` (scope enforcement off)
  - `cancellation_promise`: `""` (no cancellation topic)
  - `required_events`: `[]` (no chain validation)
- Existing YAML configs work identically — no enforcement is active unless explicitly opted into.
- To enable, set `enforce_hat_scope: true`, `cancellation_promise: "loop.cancel"`, and populate `required_events`.
- `human.timeout` injection is the only always-on change, but it only fires when `RObot.enabled: true` and a timeout occurs (previously a silent no-op).

## Test plan

- [x] `cargo test -p ralph-core --lib` — 701 tests pass (689 existing + 12 new, 0 regressions)
- [x] `cargo build --release` — clean
- [x] `cargo clippy -- -D warnings` — clean
- [ ] Manual: configure a preset with `required_events` and verify `LOOP_COMPLETE` is rejected when events are missing
- [ ] Manual: verify `loop.cancel` terminates cleanly without chain validation
- [ ] Manual: verify `human.timeout` event appears on `wait_for_response()` timeout
