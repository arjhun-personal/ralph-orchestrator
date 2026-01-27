---
status: completed
created: 2026-01-15
started: 2026-01-15
completed: 2026-01-15
notes: Fix was already applied in commit 7fb87419. Task file created after the fix.
---
# Task: Fix Metrics Extraction in evaluate-preset.sh

## Description
Fix the metrics extraction logic in `tools/evaluate-preset.sh` that incorrectly parses session.jsonl files. The grep patterns don't match the actual session JSONL format, causing all metrics to show 0 iterations, 0 events, and "unknown" hats. Additionally, the JSON output is malformed due to newline handling issues.

## Background
The preset evaluation system runs Ralph with different hat collection presets and records metrics for analysis. The `evaluate-preset.sh` script extracts metrics from `session.jsonl` files to populate `metrics.json`. However, the grep patterns were written for an older/different session format and don't match the actual recorded data.

**Current broken patterns:**
- `"type":"iteration_start"` → doesn't exist
- `"type":"event_published"` → doesn't exist
- `"type":"hat_activated"` → doesn't exist

**Actual session.jsonl format:**
```json
{"ts":1768529297249,"event":"_meta.loop_start","data":{"max_iterations":20,"prompt_file":"","ux_mode":"cli"}}
{"ts":1768529343000,"event":"bus.publish","data":{"payload":"...","source":null,"target":null,"topic":"task.resume"}}
```

The output log contains iteration markers like:
```
ITERATION 1 │ 🎭 ralph │ 0s elapsed │ 1/100
ITERATION 2 │ 🔨 builder │ 29s elapsed │ 2/100
```

## Technical Requirements
1. Update iteration count extraction to use actual session format or output.log markers
2. Update event count extraction to match `"event":"bus.publish"` pattern
3. Extract hat information from output.log iteration markers (e.g., `│ 🎭 ralph │`, `│ 🔨 builder │`)
4. Ensure metrics.json output is valid JSON (no embedded newlines in values)
5. Handle edge cases where session.jsonl or output.log may be empty or missing expected patterns

## Dependencies
- `tools/evaluate-preset.sh` - the file to modify
- `.eval/logs/*/latest/session.jsonl` - example session files to validate against
- `.eval/logs/*/latest/output.log` - example output logs to validate against

## Implementation Approach
1. Read existing session.jsonl files to confirm the actual format
2. Update ITERATIONS extraction: count `"event":"_meta.loop_start"` OR parse "ITERATION N" from output.log
3. Update EVENTS extraction: count `"event":"bus.publish"` entries
4. Add HATS extraction: parse unique hat names from output.log iteration lines (the emoji+name after `│`)
5. Fix JSON generation to properly escape/handle values and avoid newline corruption
6. Test with existing `.eval/logs/` data to verify correct extraction

## Acceptance Criteria

1. **Iteration Count Extraction**
   - Given a session.jsonl with `_meta.loop_start` events or output.log with ITERATION markers
   - When metrics are extracted
   - Then the iteration count matches the actual number of iterations run

2. **Event Count Extraction**
   - Given a session.jsonl with `bus.publish` events
   - When metrics are extracted
   - Then the event count matches the actual number of events published

3. **Hat Extraction**
   - Given an output.log with iteration lines showing hat names (e.g., `│ 🎭 ralph │`, `│ 🔨 builder │`)
   - When metrics are extracted
   - Then hats_activated contains comma-separated unique hat names

4. **Valid JSON Output**
   - Given any combination of metrics values
   - When metrics.json is generated
   - Then the file is valid JSON parseable by `jq`

5. **Edge Case Handling**
   - Given empty or missing session.jsonl/output.log files
   - When metrics extraction runs
   - Then reasonable defaults are used (0, "unknown") without script errors

6. **Backwards Compatibility**
   - Given existing evaluation logs in `.eval/logs/`
   - When re-running metrics extraction logic manually
   - Then correct metrics are extracted from historical data

## Metadata
- **Complexity**: Low
- **Labels**: Bug Fix, Evaluation, Metrics, Shell Script
- **Required Skills**: Bash scripting, grep/sed/awk, JSON handling, regex
