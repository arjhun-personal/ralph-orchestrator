# Rough Idea

I want to rethink the event-loop to be more resilient and extensible for hat collections.

## Context

Ralph is an orchestrator that wears different "hats" depending on its current role (e.g., "planner hat", "builder hat", or "no hat today—just vibing").

The current event loop (see `specs/archive/event-loop.spec.md.bak`) has:
- Core behaviors (scratchpad, specs awareness, search-first, backpressure)
- Hat-based routing (planner, builder, custom hats)
- Event-driven pub/sub architecture
- Stall recovery (retry → escalate → terminate)

## Goals

1. **More resilient** — Better recovery from failures, edge cases, and stuck states
2. **Extensible hat collections** — Easier to define and compose different hats
3. **Maintain core tenets** — Fresh context, backpressure over prescription, letting agents do the work

## Reference

The existing spec is preserved at `specs/archive/event-loop.spec.md.bak` for reference during this rethink.
