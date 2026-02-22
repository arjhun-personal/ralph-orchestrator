# Upstream PR: Fix default_publishes of completion_promise not setting completion_requested

## PR Metadata

- **Target repo**: `mikeyobrien/ralph-orchestrator`
- **Target branch**: `main`
- **Source branch**: `fix/default-publishes-completion-requested` (create from commit `ed40abe`)
- **PR title**: `fix(core): default_publishes of completion_promise must set completion_requested`
- **Depends on**: `fix/default-publishes-seen-topics` (commit `adce9bc`) — that fix added `record_topic()` to `check_default_publishes`; this fix builds on that code path

## gh command

```bash
# From your fork, create a branch and open the PR:
git checkout -b fix/default-publishes-completion-requested ed40abe
git push origin fix/default-publishes-completion-requested

gh pr create \
  --repo mikeyobrien/ralph-orchestrator \
  --head arjhun-personal:fix/default-publishes-completion-requested \
  --base main \
  --title "fix(core): default_publishes of completion_promise must set completion_requested" \
  --body-file upstream-PRs/default-publishes-completion-requested-body.md
```

---

## Context

This is the second bug in `check_default_publishes`, closely related to the `seen_topics` bug fixed in `adce9bc`. While that fix ensured default events are visible to `required_events` chain validation, this fix addresses a separate code path: the `completion_requested` flag that gates loop termination.

When a hat's `default_publishes` matches the `completion_promise` (e.g., `LOOP_COMPLETE`), the event is published to the bus but `completion_requested` is never set to `true`. This flag is only set in `process_events_from_jsonl()`, which `default_publishes` bypasses entirely. The result: the loop spins forever — the completion event exists on the bus but `check_completion_event()` never fires because the flag is `false`.

## Impact

Any preset where:
- The final hat (e.g., `final_committer`) has `default_publishes` set to the `completion_promise` topic, AND
- The agent completes its work without explicitly emitting `LOOP_COMPLETE` via JSONL

...will enter an **infinite loop** between the final hat and its predecessors. The loop cannot recover because the completion event is published to the bus (waking downstream hats) but the termination check never passes.

In practice, this manifests as iterations cycling endlessly between dispatcher → builder → dispatcher after all work is already done.

## Severity

**Critical** — this is worse than the `seen_topics` bug because there is no eventual termination. The `seen_topics` bug would be caught by `max_iterations`; this bug causes an infinite cycle of hats re-activating each other with no progression toward completion, burning API credits on redundant iterations doing zero useful work.
