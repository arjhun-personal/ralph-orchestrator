---
status: completed
created: 2026-01-15
started: 2026-01-15
completed: 2026-01-15
---
# Task: Implement Fast Path Delegation (REC-003)

## Description

Implement a "fast path" in Ralph's coordinator that skips the PLAN step when `starting_event` is configured and the scratchpad is empty/missing. This unblocks E2E preset evaluation by allowing immediate delegation to specialized hats.

## Background

Ralph's coordinator currently spends ~120s on gap analysis and planning before delegating to specialized hats, consuming the entire idle timeout. This prevents actual E2E testing of presets because hats never get activated.

The root cause is in `hatless_ralph.rs` - the multi-hat workflow always includes a PLAN step even when immediate delegation is appropriate.

## Reference Documentation

**Required:**
- `crates/ralph-core/src/hatless_ralph.rs` - The coordinator implementation
- `.agent/preset-eval-scratchpad.md` - Documents the issue and recommended fix (REC-003)

## Technical Requirements

1. Add a method to detect "fresh start" conditions:
   - `starting_event` is configured (Some)
   - Scratchpad file doesn't exist OR is empty/minimal

2. Modify prompt generation to use fast path when conditions are met:
   - Skip PLAN step entirely
   - Generate minimal prompt: "Publish `{starting_event}` immediately to start the workflow"

3. Preserve existing behavior when:
   - No `starting_event` is configured
   - Scratchpad has existing content (continuation scenario)

## Dependencies

- `std::path::Path` for scratchpad existence check
- `std::fs` for reading scratchpad content (if needed)
- Existing `HatlessRalph` struct and methods

## Implementation Approach

1. Add `is_fresh_start(&self) -> bool` method to `HatlessRalph`:
   ```rust
   fn is_fresh_start(&self) -> bool {
       if self.starting_event.is_none() {
           return false;
       }
       // Check if scratchpad exists and has meaningful content
       let path = Path::new(&self.core.scratchpad);
       if !path.exists() {
           return true;
       }
       // Optionally: check if file is empty or just has template
       true // or more sophisticated check
   }
   ```

2. Modify `workflow_section()` to check fast path:
   ```rust
   fn workflow_section(&self) -> String {
       if self.hat_topology.is_some() {
           if self.is_fresh_start() {
               // Fast path: immediate delegation
               return format!(r"## WORKFLOW

**FAST PATH**: Publish `{}` immediately to start the hat workflow.
Do not plan or analyze — delegate now.

", self.starting_event.as_ref().unwrap());
           }
           // Existing multi-hat workflow...
       }
   }
   ```

3. Add unit test for fast path behavior

## Acceptance Criteria

1. **Fast Path Triggers Correctly**
   - Given a config with `starting_event: "task.start"` and no scratchpad
   - When Ralph builds the prompt
   - Then the prompt instructs immediate delegation without PLAN step

2. **Existing Behavior Preserved**
   - Given a config with `starting_event` and existing scratchpad content
   - When Ralph builds the prompt
   - Then the prompt includes the full PLAN → DELEGATE workflow

3. **No Starting Event Unchanged**
   - Given a config without `starting_event`
   - When Ralph builds the prompt
   - Then behavior is unchanged (solo mode or multi-hat with planning)

4. **All Tests Pass**
   - Given the implementation is complete
   - When running `cargo test -p ralph-core`
   - Then all existing tests pass plus new `test_fast_path_with_starting_event`

5. **E2E Preset Evaluation Works**
   - Given the fix is applied
   - When running preset evaluation
   - Then hats are activated within seconds, not 120s

## Metadata

- **Complexity**: Low
- **Labels**: coordinator, performance, preset-evaluation, hatless-ralph
- **Required Skills**: Rust, Ralph architecture, prompt engineering
