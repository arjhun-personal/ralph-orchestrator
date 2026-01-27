---
status: completed
created: 2026-01-19
started: 2026-01-19
completed: 2026-01-19
---
# Task: Documentation Updates

## Description
Update all documentation files to replace `-i`/`--interactive` with `--tui` and update terminology from "interactive mode" to "TUI mode" or "observation mode".

## Background
Approximately 21+ markdown files reference the old `-i` flag or "interactive mode" terminology. These must be updated atomically to maintain documentation consistency.

## Reference Documentation
**Required:**
- Design: specs/tui-observation-mode/design.md (Section 7 - Documentation Updates)

**Additional References:**
- specs/tui-observation-mode/plan.md (Step 8)

**Note:** You MUST read the design document before beginning implementation.

## Technical Requirements
1. Replace `-i` with `--tui` in all command examples
2. Replace `--interactive` with `--tui` where used
3. Replace "interactive mode" terminology with "TUI mode" or "observation mode"
4. Update files in: `docs/`, `README.md`, `CLAUDE.md`, `specs/`, `presets/`

## Dependencies
- Task 07 (example and specs updated) - code changes complete

## Implementation Approach
1. Run `grep -rn "\-i\b\|--interactive" --include="*.md"` to find all files
2. For each file, read and update flag references
3. Update terminology "interactive mode" → "TUI mode"
4. Verify with grep that no old references remain
5. Spot check key files (README.md, CLAUDE.md) for correctness

## Acceptance Criteria

1. **No -i flag in docs**
   - Given documentation updates are complete
   - When running `grep -rn " -i " --include="*.md"`
   - Then no results match TUI flag context (some -i may exist for other purposes)

2. **No --interactive flag in docs**
   - Given documentation updates are complete
   - When running `grep -rn "\-\-interactive" --include="*.md"`
   - Then no results are returned

3. **--tui flag documented**
   - Given README.md is updated
   - When searching for "--tui"
   - Then flag is documented with description

4. **Terminology updated**
   - Given documentation is updated
   - When searching for "interactive mode"
   - Then term is replaced with "TUI mode" or "observation mode" where referring to Ralph's TUI

5. **CLAUDE.md updated**
   - Given CLAUDE.md is updated
   - When reading TUI validation section
   - Then examples use `--tui` flag

6. **All documentation consistent**
   - Given all files are updated
   - When reviewing documentation
   - Then no mixed terminology exists (no "interactive" for TUI references)

## Metadata
- **Complexity**: Medium
- **Labels**: documentation, breaking-change
- **Required Skills**: Markdown, grep/sed
