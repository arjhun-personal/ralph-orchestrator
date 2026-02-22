# Upstream PR: Fix default_publishes not recording topics for chain validation

## PR Metadata

- **Target repo**: `mikeyobrien/ralph-orchestrator`
- **Target branch**: `main`
- **Source branch**: `fix/default-publishes-seen-topics` (create from commit `adce9bc`)
- **PR title**: `fix(core): record default_publishes topics in seen_topics for chain validation`

## gh command

```bash
# From your fork, create a branch and open the PR:
git checkout -b fix/default-publishes-seen-topics adce9bc
git push origin fix/default-publishes-seen-topics

gh pr create \
  --repo mikeyobrien/ralph-orchestrator \
  --head arjhun-personal:fix/default-publishes-seen-topics \
  --base main \
  --title "fix(core): record default_publishes topics in seen_topics for chain validation" \
  --body-file upstream-PRs/default-publishes-seen-topics-body.md
```

---

## Context

This bug was introduced by the interaction of two features from the hat scope enforcement PR:

1. **`default_publishes`** — injects a fallback event when a hat writes no events to JSONL
2. **`required_events` chain validation** — gates `LOOP_COMPLETE` on all required topics having been seen

These features were developed together but the `default_publishes` path was not wired to `record_topic()`, creating a gap where events delivered via `default_publishes` are invisible to chain validation.

## Impact

Any preset that:
- Uses `required_events` for completion gating, AND
- Has a hat with `default_publishes` for one of those required topics, AND
- The agent doesn't explicitly emit the event in JSONL

...will enter an infinite loop that burns iterations until `max_iterations` is hit. The loop cannot recover because no hat can retroactively produce the missing event — the workflow phase that would have produced it already completed successfully.

## Severity

**High** — this is a silent resource-wasting bug. The loop appears to be working (agents run, tests pass, events fire) but can never terminate successfully. It burns API credits on redundant iterations doing no useful work.
