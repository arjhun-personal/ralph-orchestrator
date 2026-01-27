---
status: completed
created: 2026-01-15
started: 2026-01-15
completed: 2026-01-15
---
# Task: Fix JSONL Event Prompting to Prevent Multi-line Payload Failures

## Description
Update the `event_writing_section()` function in `hatless_ralph.rs` to guide agents toward writing valid JSONL events. Currently, agents write YAML-structured data with literal newlines, which breaks the JSONL format (requires single-line JSON). The fix updates prompting to: show JSON object payload examples, explicitly ban YAML formatting in payloads, and frame events as brief routing signals with detailed data going to scratchpad.

## Background
Analysis of `.agent/events.jsonl` revealed a pattern:
- **Agent-written prose/markdown content**: Properly escaped `\n` characters ✅
- **Agent-written YAML-structured content**: Literal newlines that break JSONL ❌
- **Ralph-written events**: Always properly escaped via `serde_json` ✅

The agent CAN write valid JSONL (proven by lines with complex markdown content), but fails when outputting YAML-style structured data. The `EventReader` already supports JSON object payloads via `deserialize_flexible_payload`, so the fix is purely a prompting change to steer agents away from YAML toward JSON objects or brief prose.

Example of broken event (literal newlines):
```json
{"topic": "architecture.done", "payload": "files_reviewed: [...]
concerns:
  - type: naming
    description: ...
approval: approved", "ts": "..."}
```

Example of working event (same content, escaped):
```json
{"topic": "architecture.done", "payload": "files_reviewed: [...]\nconcerns:\n  - type: naming...", "ts": "..."}
```

## Reference Documentation
**Required:**
- Source file: `crates/ralph-core/src/hatless_ralph.rs` (lines 265-275, `event_writing_section()`)
- Event reader: `crates/ralph-core/src/event_reader.rs` (shows `deserialize_flexible_payload` supporting object payloads)

**Additional References:**
- Current events file: `.agent/events.jsonl` (shows failure patterns)
- Documentation: `docs/guide/agents.md` (lines 670-710, needs alignment)
- CLAUDE.md: Ralph tenets, especially "Disk Is State, Git Is Memory"

**Note:** Read the current `event_writing_section()` implementation and examine `.agent/events.jsonl` to understand the failure patterns before implementing changes.

## Technical Requirements
1. Update `event_writing_section()` in `hatless_ralph.rs` to produce clearer prompting
2. Show JSON object payload example (already supported by EventReader)
3. Explicitly warn against YAML formatting in payloads
4. Frame events as routing signals, not data transport
5. Direct agents to use scratchpad for detailed/structured data
6. Keep the instruction concise (avoid prompt bloat)

## Dependencies
- No code dependencies - this is a prompting-only change
- `EventReader::deserialize_flexible_payload` already handles JSON object payloads
- No changes needed to event parsing or reading logic

## Implementation Approach
1. Read current `event_writing_section()` implementation (lines 265-275)
2. Review `.agent/events.jsonl` to confirm understanding of failure patterns
3. Rewrite the section to include:
   - Clear single-line JSON requirement with warning
   - JSON object payload example: `{"topic": "...", "payload": {"status": "...", "count": N}, "ts": "..."}`
   - Explicit ban on YAML formatting in payloads
   - Guidance: "Events are routing signals, not data dumps"
   - Redirect for detailed data: "Write structured results to scratchpad, emit brief event"
4. Update corresponding documentation in `docs/guide/agents.md` for consistency
5. Run `cargo test` to verify no regressions
6. Optionally: Add a unit test for the new prompt content

## Acceptance Criteria

1. **JSON Object Payload Example Present**
   - Given the updated `event_writing_section()` function
   - When the prompt is generated
   - Then it includes an example with a JSON object payload like `{"payload": {"key": "value"}}`

2. **YAML Format Warning Present**
   - Given the updated prompt
   - When an agent reads the EVENT WRITING section
   - Then it sees an explicit warning against using YAML formatting in payloads

3. **Single-Line Requirement Emphasized**
   - Given the updated prompt
   - When an agent reads the EVENT WRITING section
   - Then it clearly states that each JSON must be on ONE LINE

4. **Scratchpad Redirection Present**
   - Given the updated prompt
   - When an agent needs to write detailed/structured data
   - Then the prompt directs them to write details to scratchpad and emit a brief event

5. **Events Framed as Signals**
   - Given the updated prompt
   - When an agent reads the EVENT WRITING section
   - Then events are framed as "routing signals" not "data transport"

6. **Documentation Alignment**
   - Given the changes to `hatless_ralph.rs`
   - When reviewing `docs/guide/agents.md`
   - Then the documentation reflects the same guidance (JSON objects, no YAML, brief payloads)

7. **No Test Regressions**
   - Given the prompt changes
   - When running `cargo test`
   - Then all existing tests pass

8. **Prompt Remains Concise**
   - Given the updated `event_writing_section()`
   - When measuring the output length
   - Then the section remains reasonably concise (under 20 lines of prompt text)

## Metadata
- **Complexity**: Low
- **Labels**: Prompting, JSONL, Events, Bug Fix, Agent Guidance
- **Required Skills**: Rust string formatting, understanding of JSONL format, prompt engineering
