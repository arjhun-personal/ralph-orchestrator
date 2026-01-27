---
status: completed
created: 2026-01-14
started: 2026-01-15
completed: 2026-01-15
---
# Task: Always Use Hatless Ralph Iteration When Custom Hats Defined

## Description
Modify the event loop behavior so that when custom hats are defined in configuration, the iteration execution is always handled by Hatless Ralph rather than delegating to individual hat backends. Custom hats should still define the topology (pub/sub contracts, instructions) that Ralph uses for coordination context, but Ralph becomes the sole executor.

## Background
Currently, when custom hats are defined in configuration:
1. Custom hats are registered with the event bus with routing priority
2. Events matching a hat's triggers are routed to that specific hat's backend
3. Ralph only handles events that no custom hat subscribes to (fallback)

The desired behavior is:
1. Custom hats define topology and contracts (what events they conceptually handle)
2. Ralph is aware of this topology and includes it in prompt context
3. **All iterations are executed by Ralph** regardless of event routing
4. Hat topology serves as "documentation" for Ralph's coordination, not execution delegation

This aligns with the "hatless Ralph" philosophy where Ralph is the constant coordinator that cannot be replaced or configured away.

## Reference Documentation
**Required:**
- Design: `specs/hat-collections.spec.md` - Hat design specification
- Migration: `docs/migration/v2-hatless-ralph.md` - Hatless Ralph philosophy

**Additional References:**
- `crates/ralph-core/src/event_loop.rs` - Current event loop implementation
- `crates/ralph-core/src/hatless_ralph.rs` - HatlessRalph coordinator
- `crates/ralph-core/src/hat_registry.rs` - Hat registry and routing

**Note:** You MUST read the hat-collections spec and hatless_ralph module before implementation to understand the current architecture.

## Technical Requirements
1. Modify event loop to always use Hatless Ralph for iteration execution when custom hats exist
2. Preserve hat topology information for Ralph's prompt context (the "## HATS" section)
3. Custom hats should NOT execute via their own backends - Ralph handles all work
4. Maintain backward compatibility with solo mode (no hats defined)
5. Ensure Ralph's prompt includes full hat topology for coordination awareness
6. Update any routing logic that currently delegates to individual hat backends

## Dependencies
- `ralph-core` crate - event_loop.rs, hatless_ralph.rs, hat_registry.rs
- `ralph-proto` crate - Hat struct and event bus
- Existing test infrastructure in `crates/ralph-core/tests/`

## Implementation Approach
1. Review current event routing in `event_loop.rs` to understand delegation flow
2. Identify where hat-specific backend execution occurs
3. Modify the iteration logic to always invoke Ralph's backend while preserving hat context
4. Update `HatlessRalph::build_prompt()` to ensure hat topology is always included when hats are defined
5. Add/update tests to verify:
   - Custom hats defined → Ralph executes all iterations
   - Hat topology visible in Ralph's prompt
   - Solo mode (no hats) continues to work unchanged
6. Verify existing tests pass with the new behavior

## Acceptance Criteria

1. **Ralph Executes All Iterations With Custom Hats**
   - Given a configuration with custom hats defined
   - When an event is published that matches a custom hat's trigger
   - Then Ralph (not the custom hat's backend) executes the iteration

2. **Hat Topology Preserved in Prompt**
   - Given a configuration with custom hats defined
   - When Ralph builds its prompt for an iteration
   - Then the prompt includes the "## HATS" section with full topology table

3. **Solo Mode Unchanged**
   - Given a configuration with no hats defined (empty or omitted)
   - When the event loop runs
   - Then Ralph operates in solo mode as before (no "## HATS" section)

4. **Hat Contracts Available for Coordination**
   - Given custom hats with triggers and publishes defined
   - When Ralph executes an iteration
   - Then Ralph's prompt includes information about which events hats conceptually handle

5. **No Backend Delegation to Custom Hats**
   - Given a custom hat with a specific backend configured (e.g., `backend: gemini`)
   - When an event matching that hat's trigger is published
   - Then the iteration uses Ralph's backend, NOT the hat's configured backend

6. **Event Flow Continues Correctly**
   - Given custom hats defining a multi-step workflow via events
   - When Ralph executes iterations
   - Then events are still published and the workflow progresses through Ralph's coordination

7. **Unit Test Coverage**
   - Given the implementation changes
   - When running the test suite
   - Then new tests verify the "always hatless iteration" behavior with >90% coverage of changed code

## Metadata
- **Complexity**: Medium
- **Labels**: Event Loop, Hatless Ralph, Architecture, Coordination
- **Required Skills**: Rust, async/event-driven architecture, understanding of Ralph's hat system
