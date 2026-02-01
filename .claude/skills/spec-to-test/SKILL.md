---
name: spec-to-test
description: Parse Given-When-Then acceptance criteria from .spec.md files and generate Rust test stubs that map 1:1 to each criterion. Designed for the spec-driven workflow.
type: anthropic-skill
version: "1.0"
---

# Spec-to-Test Stub Generation

## Overview

This skill generates test stubs from `.spec.md` acceptance criteria, ensuring every Given/When/Then criterion has a corresponding test before implementation begins. It bridges the gap between specification and code in the spec-driven pipeline.

## When to Use

- At the start of any `spec.approved` → `implementation.done` phase in the spec-driven workflow
- When implementing features that have a `.spec.md` with acceptance criteria
- When the Implementer or Developer hat needs to generate test stubs from a spec

## Core Requirements

1. **1:1 Mapping**: Every Given/When/Then criterion MUST produce exactly one test stub.
2. **Stubs Fail by Default**: Generated tests MUST compile but fail (contain `todo!()` or `unimplemented!()` in the body). This enforces TDD — tests are red until implementation is complete.
3. **Convention Matching**: Test stubs MUST follow the naming and structure conventions of existing tests in the target crate.
4. **Spec Traceability**: Each test MUST include a doc comment linking back to the spec file and criterion number.

## Workflow

### 1) Locate the Spec

Find the relevant `.spec.md` file:

```bash
# List all spec files
find .ralph/specs -name "*.spec.md" | sort

# Or search for a specific spec
rg -l "keyword" .ralph/specs/
```

Read the spec and identify the **Acceptance Criteria** section.

### 2) Extract Acceptance Criteria

Parse each Given/When/Then triple from the spec. The criteria follow these formats:

**Bold markdown (most common):**
```markdown
**Given** `backend: "amp"` in config
**When** Ralph executes an iteration
**Then** both `--dangerously-allow-all` and `-x` flags are included
```

**List items:**
```markdown
- **Given** a hat collection where two hats trigger on the same event
- **When** configuration is loaded
- **Then** error "Ambiguous routing for trigger" is returned
```

**Plain text:**
```markdown
Given the server is started
When a GET request is sent
Then a 200 response is returned
```

**Note:** The `When` clause is sometimes omitted (implicit). Handle this by using the `Given` as context for the test setup and the `Then` as the assertion.

For each criterion, record:
- **Criterion number** (sequential, 1-based)
- **Given** text (precondition / setup)
- **When** text (action / trigger) — may be absent
- **Then** text (expected outcome / assertion)

### 3) Determine Test Location

Match the test location to existing crate conventions:

```bash
# Find where tests live for the relevant module
rg --files -g "crates/*/tests/*.rs"
rg -n "#\[cfg\(test\)\]" crates/
```

**Decision rules:**
- If the spec maps to a single module → add inline `#[cfg(test)]` tests
- If the spec spans multiple modules → create an integration test in `crates/<crate>/tests/`
- If the spec is for CLI behavior → use `crates/ralph-cli/tests/`
- Mirror the test helper patterns from nearby tests

### 4) Generate Test Stubs

For each criterion, generate a test stub:

```rust
/// Spec: <spec-filename> — Criterion #<N>
/// Given <given text>
/// When <when text>
/// Then <then text>
#[test]
fn <spec_name>_criterion_<N>_<descriptive_slug>() {
    // Setup: <given text>
    // TODO: Set up preconditions

    // Act: <when text>
    // TODO: Perform the action

    // Assert: <then text>
    todo!("Implement: <then text>");
}
```

**Naming convention**: `<spec_name>_criterion_<N>_<short_description>`

Example from `amp.spec.md`:

```rust
/// Spec: amp.spec.md — Criterion #1
/// Given `backend: "amp"` in config
/// When Ralph executes an iteration
/// Then both `--dangerously-allow-all` and `-x` flags are included
#[test]
fn amp_criterion_1_includes_headless_flags() {
    // Setup: backend "amp" in config
    // TODO: Create config with backend: "amp"

    // Act: Ralph executes an iteration
    // TODO: Build the command

    // Assert: both flags are included
    todo!("Verify --dangerously-allow-all and -x flags are present");
}

/// Spec: amp.spec.md — Criterion #2
/// Given `backend: "amp"` in config
/// When Ralph builds the command
/// Then the prompt is passed as an argument to `-x`, not via stdin
#[test]
fn amp_criterion_2_prompt_via_argument() {
    // Setup: backend "amp" in config
    // TODO: Create config with backend: "amp"

    // Act: Ralph builds the command
    // TODO: Build the command with a prompt

    // Assert: prompt passed as argument to -x
    todo!("Verify prompt is an argument to -x, not stdin");
}
```

**For async tests** (when the module uses async):

```rust
#[tokio::test]
async fn <spec_name>_criterion_<N>_<descriptive_slug>() {
    todo!("Implement criterion");
}
```

### 5) Verify Stubs Compile

Run the test suite to confirm stubs compile (they should fail, not error):

```bash
# Compile only — don't run
cargo test --no-run -p <crate>

# Run to confirm failures are todo!() panics
cargo test -p <crate> <test_name> 2>&1 | head -20
```

Every stub should produce: `thread 'test_name' panicked at 'not yet implemented: ...'`

### 6) Report Generated Stubs

After generating stubs, output a summary:

```
Generated test stubs from <spec-file>:
  - <test_name_1> (criterion #1: <short description>)
  - <test_name_2> (criterion #2: <short description>)
  ...
Total: <N> test stubs from <M> acceptance criteria
Location: <file path>
Status: All stubs compile, all fail with todo!()
```

## Programmatic Support

The `ralph-core` crate provides a Rust API for extracting criteria programmatically:

```rust
use ralph_core::preflight::{
    extract_acceptance_criteria,
    extract_criteria_from_file,
    extract_all_criteria,
    AcceptanceCriterion,
};

// From content string
let criteria: Vec<AcceptanceCriterion> = extract_acceptance_criteria(content);
for c in &criteria {
    println!("Given: {}", c.given);
    if let Some(when) = &c.when {
        println!("When: {}", when);
    }
    println!("Then: {}", c.then);
}

// From a single file (skips status: implemented)
let criteria = extract_criteria_from_file(Path::new(".ralph/specs/feature.spec.md"));

// From all specs in a directory
let all = extract_all_criteria(Path::new(".ralph/specs/"))?;
for (filename, criteria) in &all {
    println!("{}: {} criteria", filename, criteria.len());
}
```

## Integration with spec-driven.yml

This skill is referenced by the **Implementer** hat in `presets/spec-driven.yml`. The workflow:

1. `spec.approved` triggers the Implementer
2. Implementer uses this skill to generate test stubs from the approved spec
3. Implementer then implements code to make stubs pass (TDD)
4. `implementation.done` triggers the Verifier, who checks criteria satisfaction

## Anti-Patterns (Avoid)

- Writing implementation code before generating test stubs from the spec
- Generating tests that pass without implementation (tests should fail with `todo!()`)
- Creating test stubs that don't map 1:1 to spec criteria
- Skipping the compilation check after generating stubs
- Adding assertions beyond what the spec criterion requires

## Checklist

- [ ] Spec file located and acceptance criteria identified
- [ ] All Given/When/Then triples extracted (count matches spec)
- [ ] Test stubs generated with 1:1 mapping to criteria
- [ ] Each stub has spec traceability doc comment
- [ ] Each stub follows crate test conventions
- [ ] All stubs compile successfully
- [ ] All stubs fail with `todo!()` (red phase of TDD)
- [ ] Summary of generated stubs reported
