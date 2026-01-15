# Hat Collection Preset Benchmark Specification

> Defines a repeatable methodology for testing and benchmarking hat collection presets across Ralph releases, backends, and LLM versions.

## Overview

### Purpose

Establish a standardized evaluation framework to:
1. **Validate** preset correctness after changes
2. **Benchmark** performance across backends (Kiro, Claude, Gemini)
3. **Track** quality improvements over time
4. **Detect** regressions before release

### Scope

- 12 hat collection presets in `presets/`
- Multiple backends: Kiro CLI, Claude CLI, Gemini CLI
- Metrics: correctness, timing, token usage, error rates

---

## Evaluation Dimensions

### 1. Correctness

| Criterion | Pass Condition |
|-----------|----------------|
| **Hat Flow** | Events route to expected hats in expected order |
| **Event Publishing** | Each hat publishes declared events |
| **Completion** | Reaches `LOOP_COMPLETE` appropriately |
| **Instructions** | Hat follows its instructions |
| **No Loops** | No infinite event cycles |

### 2. Performance

| Metric | Measurement |
|--------|-------------|
| **Total Time** | Wall clock from start to completion |
| **Time per Hat** | Average time in each hat |
| **Iterations** | Number of event loop iterations |
| **Token Usage** | Input/output tokens per hat (if available) |

### 3. Quality

| Metric | Measurement |
|--------|-------------|
| **Task Completion** | Did the preset accomplish the test task? |
| **Output Quality** | LLM-as-judge score (1-5) on deliverables |
| **Instruction Adherence** | % of instruction points followed |

### 4. Reliability

| Metric | Measurement |
|--------|-------------|
| **Success Rate** | % of runs that complete without errors |
| **Error Types** | Classification of failures |
| **Recovery** | Does the preset recover from transient errors? |

---

## Test Tasks

Each preset has a canonical test task designed to exercise its workflow:

| Preset | Test Task | Complexity | Expected Iterations |
|--------|-----------|------------|---------------------|
| `tdd-red-green.yml` | Implement `is_palindrome()` | Simple | 3-6 |
| `adversarial-review.yml` | Review input sanitization | Medium | 3-5 |
| `socratic-learning.yml` | Explain `HatRegistry` | Simple | 3-9 |
| `spec-driven.yml` | Specify `truncate()` function | Medium | 4-8 |
| `mob-programming.yml` | Implement `Stack` struct | Simple | 3-6 |
| `scientific-method.yml` | Debug assertion failure | Medium | 4-8 |
| `code-archaeology.yml` | Analyze `config.rs` history | Medium | 4-5 |
| `performance-optimization.yml` | Optimize hash lookup | Complex | 4-8 |
| `api-design.yml` | Design `Cache` interface | Medium | 4-6 |
| `documentation-first.yml` | Document `RateLimiter` | Medium | 4-6 |
| `incident-response.yml` | Respond to CI failure | Medium | 4-5 |
| `migration-safety.yml` | Plan config migration | Complex | 4-5 |

---

## Execution Protocol

### Prerequisites

```bash
# Verify tools are installed
ralph --version
kiro-cli --version
claude --version  # optional

# Verify presets exist
ls presets/*.yml | wc -l  # Should be 12+

# Create evaluation workspace
mkdir -p .eval/{sandbox,logs,results}
```

### Single Preset Evaluation

```bash
#!/bin/bash
# evaluate-preset.sh <preset-name> <backend>

PRESET=$1
BACKEND=${2:-kiro}
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_DIR=".eval/logs/${PRESET}/${TIMESTAMP}"
mkdir -p "$LOG_DIR"

# Load test task from mapping
TEST_TASK=$(yq '.test_tasks["'$PRESET'"]' tools/preset-test-tasks.yml)

# Run evaluation
time ralph run \
  -c "presets/${PRESET}.yml" \
  -a "$BACKEND" \
  -p "$TEST_TASK" \
  --record-session "$LOG_DIR/session.jsonl" \
  2>&1 | tee "$LOG_DIR/output.log"

# Capture exit code
echo $? > "$LOG_DIR/exit_code"

# Extract metrics
ralph analyze-session "$LOG_DIR/session.jsonl" > "$LOG_DIR/metrics.json"
```

### Full Suite Evaluation

```bash
#!/bin/bash
# evaluate-all-presets.sh <backend>

BACKEND=${1:-kiro}
SUITE_ID=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR=".eval/results/${SUITE_ID}"
mkdir -p "$RESULTS_DIR"

PRESETS=(
  "tdd-red-green"
  "adversarial-review"
  "socratic-learning"
  "spec-driven"
  "mob-programming"
  "scientific-method"
  "code-archaeology"
  "performance-optimization"
  "api-design"
  "documentation-first"
  "incident-response"
  "migration-safety"
)

for preset in "${PRESETS[@]}"; do
  echo "=== Evaluating: $preset ==="
  ./tools/evaluate-preset.sh "$preset" "$BACKEND"

  # Copy results
  cp ".eval/logs/${preset}/latest/metrics.json" "$RESULTS_DIR/${preset}.json"
done

# Generate summary
./tools/generate-benchmark-report.sh "$RESULTS_DIR" > "$RESULTS_DIR/SUMMARY.md"
```

---

## Metrics Collection

### Session Recording Format

Ralph's `--record-session` produces JSONL with:

```jsonl
{"type": "iteration_start", "iteration": 1, "timestamp": "..."}
{"type": "hat_activated", "hat_id": "test_writer", "trigger": "task.start"}
{"type": "cli_start", "backend": "kiro", "prompt_length": 1234}
{"type": "cli_complete", "duration_ms": 45000, "output_length": 5678}
{"type": "event_published", "topic": "test.written", "from_hat": "test_writer"}
{"type": "iteration_end", "iteration": 1, "duration_ms": 46000}
```

### Derived Metrics

```python
# metrics.py - Extract metrics from session JSONL

def extract_metrics(session_path: str) -> dict:
    return {
        "total_iterations": count_iterations(session),
        "total_duration_ms": sum_durations(session),
        "hats_activated": list_unique_hats(session),
        "events_published": list_events(session),
        "errors": extract_errors(session),
        "completion_status": check_completion(session),
        "hat_timings": {
            hat: avg_duration(session, hat)
            for hat in unique_hats(session)
        }
    }
```

---

## Quality Scoring

### LLM-as-Judge Rubric

For subjective quality assessment, use this rubric:

```yaml
# quality-rubric.yml

task_completion:
  5: "Task fully completed, all acceptance criteria met"
  4: "Task mostly completed, minor gaps"
  3: "Task partially completed, significant gaps"
  2: "Task attempted but largely failed"
  1: "Task not meaningfully attempted"

instruction_adherence:
  5: "All hat instructions followed precisely"
  4: "Most instructions followed, minor deviations"
  3: "Some instructions followed, notable deviations"
  2: "Few instructions followed"
  1: "Instructions largely ignored"

workflow_quality:
  5: "Clean event flow, appropriate hand-offs"
  4: "Mostly clean flow, minor issues"
  3: "Flow works but with friction"
  2: "Flow is confusing or error-prone"
  1: "Flow is broken"
```

### Automated Quality Check

```bash
# Uses /tui-validate skill pattern for LLM-as-judge
ralph judge \
  --input "$LOG_DIR/session.jsonl" \
  --rubric "tools/quality-rubric.yml" \
  --output "$LOG_DIR/quality-scores.json"
```

---

## Benchmark Comparison

### Cross-Backend Comparison

```markdown
| Preset | Kiro Time | Claude Time | Gemini Time | Winner |
|--------|-----------|-------------|-------------|--------|
| tdd-red-green | 45s | 38s | 52s | Claude |
| adversarial-review | 62s | 55s | 70s | Claude |
...
```

### Cross-Version Comparison

```markdown
| Preset | v2.0.0 | v2.1.0 | Delta |
|--------|--------|--------|-------|
| tdd-red-green | 45s | 42s | -7% |
| adversarial-review | 62s | 58s | -6% |
...
```

### Regression Detection

A preset is considered **regressed** if:
- Success rate drops by >10%
- Average time increases by >20%
- Quality score drops by >0.5 points

---

## CI Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/preset-benchmark.yml

name: Preset Benchmark

on:
  pull_request:
    paths:
      - 'presets/**'
      - 'crates/ralph-core/src/hat_registry.rs'
  schedule:
    - cron: '0 0 * * 0'  # Weekly

jobs:
  benchmark:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        backend: [kiro, claude]
        preset: [tdd-red-green, spec-driven, scientific-method]

    steps:
      - uses: actions/checkout@v4

      - name: Setup Ralph
        run: cargo build --release

      - name: Run Benchmark
        run: |
          ./tools/evaluate-preset.sh ${{ matrix.preset }} ${{ matrix.backend }}

      - name: Check Regression
        run: |
          ./tools/check-regression.sh ${{ matrix.preset }} ${{ matrix.backend }}

      - name: Upload Results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-${{ matrix.preset }}-${{ matrix.backend }}
          path: .eval/logs/${{ matrix.preset }}/latest/
```

---

## Reporting

### Summary Report Template

```markdown
# Preset Benchmark Report

**Date**: 2025-01-14
**Ralph Version**: 2.0.0
**Backend**: Kiro CLI v1.25

## Overall Results

| Status | Count | Percentage |
|--------|-------|------------|
| ✅ Pass | 10 | 83% |
| ⚠️ Partial | 1 | 8% |
| ❌ Fail | 1 | 8% |

## Performance Summary

| Metric | Min | Avg | Max | P95 |
|--------|-----|-----|-----|-----|
| Duration (s) | 32 | 58 | 124 | 98 |
| Iterations | 3 | 5.2 | 9 | 8 |
| Tokens (K) | 12 | 28 | 56 | 48 |

## Regressions Detected

| Preset | Metric | Previous | Current | Delta |
|--------|--------|----------|---------|-------|
| spec-driven | duration | 45s | 68s | +51% ⚠️ |

## Recommendations

1. Investigate spec-driven duration regression
2. Consider reducing iteration count for socratic-learning
3. Add timeout handling for performance-optimization
```

---

## Acceptance Criteria

### Given-When-Then

```gherkin
Feature: Preset Benchmark Execution

Scenario: Successful preset evaluation
  Given a preset file exists at presets/<name>.yml
  And the backend CLI is available
  When I run the evaluation script
  Then the preset completes without errors
  And metrics are recorded to the log directory
  And a quality score is generated

Scenario: Regression detection
  Given a baseline benchmark exists
  And I run a new benchmark
  When the duration increases by >20%
  Then a regression warning is generated
  And the CI check fails

Scenario: Cross-backend comparison
  Given benchmarks for multiple backends
  When I generate the comparison report
  Then each preset shows timings for each backend
  And a winner is identified per preset
```

---

## Files

| File | Purpose |
|------|---------|
| `ralph.preset-evaluator.yml` | Meta-ralph config for evaluation |
| `tools/PRESET_EVALUATOR_PROMPT.md` | Instructions for the evaluator agent |
| `tools/preset-evaluation-findings.md` | Manual findings document |
| `tools/evaluate-preset.sh` | Single preset evaluation script |
| `tools/evaluate-all-presets.sh` | Full suite evaluation script |
| `tools/preset-test-tasks.yml` | Mapping of presets to test tasks |
| `tools/quality-rubric.yml` | LLM-as-judge scoring rubric |
| `tools/check-regression.sh` | Regression detection script |
| `.eval/` | Evaluation workspace (gitignored) |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-01-14 | Initial specification |
