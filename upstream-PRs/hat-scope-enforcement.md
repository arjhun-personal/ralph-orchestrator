# Upstream PR: Hat Scope Enforcement, Event Chain Validation, and Human Timeout Routing

## PR Metadata

- **Target repo**: `mikeyobrien/ralph-orchestrator`
- **Target branch**: `main`
- **Source branch**: `hat-enforcement` (create from commit `159821d`)
- **PR title**: `feat(core): add hat scope enforcement, event chain validation, and human timeout routing`

## gh command

```bash
# From your fork, create a branch and open the PR:
git checkout -b hat-enforcement 159821d
git push origin hat-enforcement

gh pr create \
  --repo mikeyobrien/ralph-orchestrator \
  --head arjhun-personal:hat-enforcement \
  --base main \
  --title "feat(core): add hat scope enforcement, event chain validation, and human timeout routing" \
  --body-file upstream-PRs/hat-scope-enforcement-body.md
```

---

## PR Body

_(Also saved separately as `hat-scope-enforcement-body.md` for `--body-file` usage)_

### Summary

Three layers of defense-in-depth to prevent agents from bypassing hat workflow constraints. Addresses a failure mode where an agent skipped human approval, never created tasks, and implemented all phases in a single context window by emitting events outside its hat's declared `publishes` list.

- **Hat scope enforcement**: `HatRegistry::can_publish()` gates events in `process_events_from_jsonl()` against the active hat's declared `publishes` patterns. Out-of-scope events are dropped and replaced with `{hat_id}.scope_violation` diagnostic events. Ralph in coordination mode (no active hat) retains unrestricted publishing.

- **Event chain validation + `loop.cancel`**: New `required_events` config field and `seen_topics` state tracking. `check_completion_event()` becomes a hard gate — `LOOP_COMPLETE` is rejected unless all required events have been seen during the loop's lifetime. A separate `loop.cancel` event provides clean early termination (human rejection, timeout) **without** chain validation. On rejection, a `task.resume` event is injected with the missing events listed.

- **Human timeout routing**: `wait_for_response()` timeout now publishes a `human.timeout` event instead of silently continuing. This makes timeouts visible in the event log and routable to hats that declare `human.timeout` as a trigger.

### Changes

| File | Lines | What changed |
|------|-------|-------------|
| `crates/ralph-core/src/hat_registry.rs` | +77 | `can_publish()` method + 4 unit tests |
| `crates/ralph-core/src/config.rs` | +18 | `required_events`, `cancellation_promise` fields on `EventLoopConfig` |
| `crates/ralph-core/src/event_loop/loop_state.rs` | +21 | `seen_topics`, `cancellation_requested` fields + helper methods |
| `crates/ralph-core/src/event_loop/mod.rs` | +127/-4 | Scope filtering, `loop.cancel` detection, topic recording, chain validation gate, `check_cancellation_event()`, `Cancelled` variant, `human.timeout` injection |
| `crates/ralph-core/src/event_loop/tests.rs` | +477 | 21 new tests (scope, chain validation, cancellation, timeout) |
| `crates/ralph-cli/src/loop_runner.rs` | +27 | Wire `check_cancellation_event()` before completion check, `Cancelled` in match arms |
| `crates/ralph-cli/src/display.rs` | +1 | `Cancelled` display variant |
| `crates/ralph-core/src/summary_writer.rs` | +3 | `Cancelled` status text + test fixture field |
| `crates/ralph-bench/src/main.rs` | +1 | `Cancelled` format variant |

**Total: +748/-4 across 9 files**

### Design decisions

1. **Scope enforcement runs before validation** — out-of-scope events are partitioned out before the existing `build.done`/`review.done`/`verify.passed` backpressure checks, so an unauthorized `build.done` from a non-Builder hat is dropped before it can be rejected for missing evidence.

2. **`loop.cancel` is separate from `LOOP_COMPLETE`** — cancellation means "abort gracefully" and bypasses chain validation (because the workflow hasn't completed). Completion means "all work finished" and is gated. This distinction prevents legitimate abort paths (human rejection, timeout escalation) from being blocked by missing required events.

3. **`Cancelled` exit code is 0** — the loop stopped intentionally, not due to an error. But `is_success()` returns `false` because work was not completed. Callers can distinguish success from cancellation by checking the termination reason.

4. **`required_events` is a flat presence check, not a DAG** — order doesn't matter, only that each topic was seen at least once across all iterations. This is simpler than topology-derived inference and sufficient for the use case.

5. **`human.timeout` replaces silent continuation** — previously, `wait_for_response()` timeout logged a warning and continued with no event. Now it injects `human.timeout` through the same `response_event` mechanism as `human.response`, making it routable to any hat that subscribes.

### Backward compatibility

- All new config fields have defaults (`required_events: []`, `cancellation_promise: "loop.cancel"`). Existing YAML configs work unchanged.
- **Breaking for agents**: Events emitted outside a hat's `publishes` list are now dropped. This is intentional — it enforces the hat boundary contract.
- Empty `required_events` (the default) preserves existing behavior — no chain gating.
- `loop.cancel` is a new topic; no existing config uses it, so existing loops are unaffected.

### Test plan

- [x] `cargo test -p ralph-core --lib` — 701 tests pass (689 existing + 12 new, 0 regressions)
- [x] `cargo build --release` — clean
- [x] `cargo clippy -- -D warnings` — clean
- [ ] Manual: configure a preset with `required_events` and verify `LOOP_COMPLETE` is rejected when events are missing
- [ ] Manual: verify `loop.cancel` terminates cleanly without chain validation
- [ ] Manual: verify `human.timeout` event appears in the event log on `wait_for_response()` timeout

### New tests added

**Hat scope (`hat_registry.rs`):**
- `test_can_publish_allows_declared_topic`
- `test_can_publish_rejects_undeclared_topic`
- `test_can_publish_allows_wildcard`
- `test_can_publish_unknown_hat_allows_all`

**Scope enforcement (`event_loop/tests.rs`):**
- `test_scope_enforcement_drops_unauthorized_event`
- `test_scope_enforcement_allows_authorized_event`
- `test_scope_enforcement_skipped_when_no_active_hats`
- `test_scope_violation_event_published`

**Chain validation (`event_loop/tests.rs`):**
- `test_chain_validation_rejects_completion_without_required_events`
- `test_chain_validation_accepts_completion_with_all_required_events`
- `test_chain_validation_tracks_topics_across_iterations`
- `test_chain_validation_empty_required_events_allows_completion`
- `test_chain_validation_injects_task_resume_on_rejection`

**Cancellation (`event_loop/tests.rs`):**
- `test_loop_cancel_terminates_without_chain_validation`
- `test_loop_cancel_exit_code_is_zero`
- `test_loop_cancel_is_not_success`
- `test_loop_cancel_takes_priority_over_completion`
- `test_loop_cancel_disabled_when_empty_string`

**Human timeout (`event_loop/tests.rs`):**
- `test_human_timeout_injects_timeout_event`
- `test_human_response_still_works`

### Future work (out of scope)

- Per-hat interaction timeout overrides (`interaction_timeout` on `HatConfig`)
- `ralph tools interact document` CLI command for file attachments
- Topology-derived required events (auto-infer from hat DAG)
- E2E scenario test for hat enforcement
