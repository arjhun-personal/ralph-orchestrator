# Upstream PR: Add disallowed_tools, stale loop detection, and file-modification audit

## PR Metadata

- **Target repo**: `mikeyobrien/ralph-orchestrator`
- **Target branch**: `main`
- **Source branch**: `feat/loop-cycling-phase2` (create from commit `3998962`)
- **PR title**: `feat(core): add disallowed_tools, stale loop detection, and file-modification audit`
- **Depends on**: None (builds on existing `enforce_hat_scope`, `max_activations`, `default_publishes` infrastructure)

## gh command

```bash
# From your fork, create a branch and open the PR:
git checkout -b feat/loop-cycling-phase2 3998962
git push origin feat/loop-cycling-phase2

gh pr create \
  --repo mikeyobrien/ralph-orchestrator \
  --head arjhun-personal:feat/loop-cycling-phase2 \
  --base main \
  --title "feat(core): add disallowed_tools, stale loop detection, and file-modification audit" \
  --body-file upstream-PRs/loop-cycling-phase2-body.md
```

---

## Context

This is Phase 2 of the loop cycling fix plan. Phase 1 addressed the issue through preset YAML changes (stronger dispatcher instructions, `build.noop` escape hatch, `max_activations` safety nets). Phase 2 adds robust engine-level mechanisms to prevent two classes of bugs observed in production:

1. **Hat role violations**: Dispatcher hat implementing code despite "don't build anything" instructions — because there's no mechanism to restrict tool usage per hat.
2. **Infinite cycling**: After all work is done, the loop cycles between hats emitting the same events repeatedly, wasting API credits.

## Impact

These three features provide layered defense:

- **disallowed_tools** (2A): Soft enforcement via prominent prompt section. Significantly reduces LLM tool misuse compared to buried "DON'T" instructions.
- **Stale loop detection** (2B): Hard termination when the same topic appears 3+ times consecutively. Would have saved ~$1.7 in the observed incident (5 wasted iterations).
- **File-modification audit** (2C): Post-iteration detection that emits `<hat>.scope_violation` events. Presets can route these to trigger corrective action.

## Severity

**Medium** — Phase 1 preset fixes prevent the immediate cycling bug. Phase 2 provides systemic protection against the class of bugs.
