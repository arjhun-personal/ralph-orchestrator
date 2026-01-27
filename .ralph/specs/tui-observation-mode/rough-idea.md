# TUI Observation Mode - Rough Idea

## Original Request
TUI observation mode: rename -i to --tui, remove interactive controls, keep visual monitoring

## Initial Understanding
The TUI is currently enabled via `-i/--interactive` flag and includes full interactive controls (pause, skip, abort). The request is to:
1. Rename the flag from `-i` to `--tui`
2. Remove interactive controls
3. Keep the visual monitoring (header/footer chrome)

## Key Semantic Shift
- **Current**: TUI = interactive mode (user can control execution)
- **Desired**: TUI = observation mode (user watches, doesn't control)

## Questions to Clarify
- Backward compatibility with `-i` flag?
- Which controls to remove vs keep (scroll, search)?
- Future interactive mode plans?
