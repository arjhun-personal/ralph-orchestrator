# Idea Honing

Requirements clarification for the resilient, extensible event loop with hat collections.

---

## Q1: What's the core architectural change you're envisioning?

**Answer:**

The shift is from "Ralph wears different hats" to "Ralph delegates to hat-wearing agents":

**Current design (brittle):**
- Planner and Builder are both "Ralph with a hat"
- Users can override/replace these hats
- This breaks the event graph (events published with no subscriber)
- Ralph can "forget" things

**Proposed design (resilient):**
- Single, irreplaceable "hatless Ralph" — the classic Ralph Wiggum technique
- Hatless Ralph is always present as the orchestrator/manager/scrum master
- Additional hats are optional extensions that Ralph can **delegate to**
- Users ADD hats, they don't REPLACE core Ralph
- Ralph coordinates; hats execute

**Key insight:** Ralph becomes the constant, the orchestrator. Hats become his team.

**Evidence from presets:**
- `review.yml`: `reviewer` triggers on `task.start` — no planner, coordination embedded in reviewer
- `feature.yml`: `planner` is just another replaceable hat
- Each preset rebuilds coordination from scratch
- No safety net for orphaned events

**Root cause:** Coordination is embedded in hats, not separated from them.

---

## Q2: How should hatless Ralph work in practice?

