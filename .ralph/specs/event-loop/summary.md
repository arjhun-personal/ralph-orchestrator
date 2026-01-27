# Hatless Ralph: Event Loop Redesign Summary

## Project Structure

```
specs/event-loop/
├── rough-idea.md              # Initial concept
├── idea-honing.md             # Requirements Q&A (12 questions answered)
├── summary.md                 # This document
├── research/
│   ├── current-implementation.md   # Analysis of existing code
│   └── per-hat-backends.md         # Backend flexibility + adapter docs
├── design/
│   └── detailed-design.md          # Full architecture and specs
└── implementation/
    └── plan.md                     # 12-step implementation plan
```

## Overview

This project redesigns Ralph's event loop to be more resilient and extensible through:

1. **Hatless Ralph** — A constant, irreplaceable coordinator
2. **JSONL Events** — Disk-based event publishing instead of XML parsing
3. **Per-Hat Backends** — Mix Claude, Kiro, Gemini, Codex, Amp per hat
4. **Default Publishes** — Fallback events when hats forget to publish

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Ralph is constant | Can't be misconfigured away; always catches orphaned events |
| Events on disk | Structured JSONL; no fragile XML parsing from output |
| Per-hat backends | Leverage each tool's strengths (Claude for coding, Kiro for MCP) |
| Sequential execution | KISS — parallel adds complexity without clear benefit |
| Break from planner hat | Clean break; Ralph IS the planner now |

## Architecture Highlights

```
👑 HATLESS RALPH (always present)
    │
    ├─► Owns scratchpad (.agent/scratchpad.md)
    ├─► Owns completion (LOOP_COMPLETE)
    ├─► Universal fallback for unhandled events
    │
    └─► Delegates to hats:
            │
            ├─► 🔨 Builder (backend: claude)
            ├─► 👀 Reviewer (backend: gemini)
            └─► 🔍 Researcher (backend: kiro + agent)
```

## Implementation Plan

12 incremental steps, each producing working, demoable functionality:

1. Add `HatBackend` enum and config parsing
2. Create `EventReader` for JSONL event parsing
3. Create `HatlessRalph` struct and prompt builder
4. Modify `HatRegistry` to remove default hats
5. Update `InstructionBuilder` with `build_hatless_ralph()`
6. Modify `EventLoop` to use Ralph as fallback
7. Implement `default_publishes` fallback logic
8. Add per-hat backend resolution
9. Update presets (remove planner hat)
10. Create mock CLI test harness
11. Write E2E scenario tests
12. Update documentation and migration guide

## Testing Strategy

- **Unit tests** for each new component
- **Integration tests** for core behaviors
- **E2E scenario tests** with mock CLI backend
- **Scripted YAML scenarios** for deterministic testing

## Next Steps

1. Review the detailed design at `design/detailed-design.md`
2. Check the implementation plan at `implementation/plan.md`
3. Begin implementation following the checklist

## Related Documents

- **Archived spec:** `specs/archive/event-loop.spec.md.bak`
- **Hat collections spec:** `specs/hat-collections.spec.md`
- **Adapter specs:** `specs/adapters/`
