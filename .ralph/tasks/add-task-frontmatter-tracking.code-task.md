---
status: completed
created: 2025-01-15
started: 2026-01-15
completed: 2026-01-15
---
# Task: Add Task Frontmatter Status Tracking

## Description
Add YAML frontmatter to code task files to track task lifecycle status (pending, in_progress, completed) with timestamps. The `code-task-generator` skill should create tasks with initial frontmatter, and the `code-assist` skill should automatically update status when starting and completing work.

## Background
Currently, code task files have no structured metadata for tracking their state. This makes it difficult to see at a glance which tasks are pending, in progress, or completed. Adding frontmatter enables:
- Quick status visibility when viewing task files
- Potential for task dashboards and reporting
- Audit trail of when work started and completed

## Technical Requirements
1. Define standard frontmatter format with status, created, started, completed fields
2. Update `code-task-generator` skill to add frontmatter to generated tasks
3. Update `code-assist` skill to update frontmatter on task start
4. Update `code-assist` skill to update frontmatter on task completion
5. Handle edge cases (missing frontmatter, malformed frontmatter)

## Dependencies
- YAML frontmatter parsing (regex-based is sufficient for simple structure)
- Date formatting (ISO 8601: YYYY-MM-DD)

## Implementation Approach

### Step 1: Update code-task-generator Skill
In `.claude/skills/code-task-generator/SKILL.md`:

1. Update the "Code Task Format Specification" section to include frontmatter:
```yaml
---
status: pending
created: YYYY-MM-DD
started: null
completed: null
---
```

2. Add constraint in "Generate Tasks" step: "You MUST add frontmatter with status: pending, created: <current date>, started: null, completed: null"

3. Update the example task to include frontmatter

### Step 2: Update code-assist Skill
In `.claude/skills/code-assist/SKILL.md`:

1. Add a new step or constraint at workflow start: "If the task file has frontmatter, update status to in_progress and set started to current date"

2. Add a new step or constraint at workflow end: "If the task file has frontmatter, update status to completed and set completed to current date"

3. Add helper guidance for frontmatter update:
```markdown
To update frontmatter, use the Edit tool to replace the status and date fields:
- Change `status: pending` to `status: in_progress`
- Change `started: null` to `started: YYYY-MM-DD`
```

### Step 3: Handle Edge Cases
- If task file has no frontmatter: Skip status updates (don't fail)
- If frontmatter is malformed: Log warning but continue with task
- If status is already completed: Warn but allow re-running if explicitly requested

## Acceptance Criteria

1. **Generated Tasks Have Frontmatter**
   - Given a user runs `/code-task-generator` with a description
   - When the task file is created
   - Then it includes frontmatter with `status: pending`, `created: <today>`, `started: null`, `completed: null`

2. **Status Updates on Task Start**
   - Given a pending task file with frontmatter
   - When `/code-assist` begins working on the task
   - Then frontmatter is updated to `status: in_progress` and `started: <today>`

3. **Status Updates on Task Completion**
   - Given an in_progress task file with frontmatter
   - When `/code-assist` successfully completes the task
   - Then frontmatter is updated to `status: completed` and `completed: <today>`

4. **Graceful Handling of Missing Frontmatter**
   - Given a task file without frontmatter
   - When `/code-assist` runs on the task
   - Then the task executes normally without errors (status updates are skipped)

5. **Frontmatter Format Validation**
   - Given the frontmatter format specification
   - When reviewing generated tasks
   - Then frontmatter uses ISO 8601 date format (YYYY-MM-DD) and valid YAML syntax

6. **Example Task Updated**
   - Given the code-task-generator skill documentation
   - When viewing the example task
   - Then it includes the frontmatter block demonstrating the format

## Metadata
- **Complexity**: Low
- **Labels**: Skills, Workflow, Metadata, Developer Experience
- **Required Skills**: YAML, Markdown, Skill authoring
