# Implementation Plan: Hatless Ralph

## Checklist

- [x] **Step 1:** Add `HatBackend` enum and config parsing
- [x] **Step 2:** Create `EventReader` for JSONL event parsing
- [x] **Step 3:** Create `HatlessRalph` struct and prompt builder
- [x] **Step 4:** Modify `HatRegistry` to remove default hats
- [x] **Step 5:** Update `InstructionBuilder` with `build_hatless_ralph()`
- [x] **Step 6:** Modify `EventLoop` to use Ralph as fallback
- [x] **Step 7:** Implement `default_publishes` fallback logic
- [x] **Step 8:** Add per-hat backend resolution
- [x] **Step 9:** Update presets (remove planner hat)
- [x] **Step 10:** Create mock CLI test harness
- [x] **Step 11:** Write E2E scenario tests
- [x] **Step 12:** Update documentation and migration guide

---

## Step 1: Add `HatBackend` enum and config parsing

**Objective:** Enable per-hat backend configuration with named, Kiro agent, and custom backends.

**Files to modify:**
- `crates/ralph-core/src/config.rs`

**Implementation guidance:**
1. Add `HatBackend` enum with three variants: `Named`, `KiroAgent`, `Custom`
2. Add `backend: Option<HatBackend>` field to `HatConfig`
3. Add `default_publishes: Option<String>` field to `HatConfig`
4. Implement serde deserialization with `#[serde(untagged)]` for flexible YAML syntax
5. Add `to_cli_backend()` method for resolution

**Test requirements:**
- Parse `backend: "claude"` as `Named("claude")`
- Parse `backend: { type: "kiro", agent: "builder" }` as `KiroAgent`
- Parse `backend: { command: "...", args: [...] }` as `Custom`
- Hat without `backend` deserializes as `None`

**Demo:** Config with per-hat backends loads successfully, `cargo test` passes.

---

## Step 2: Create `EventReader` for JSONL event parsing

**Objective:** Replace XML parsing with JSONL file reading for event detection.

**Files to create:**
- `crates/ralph-core/src/event_reader.rs`

**Files to modify:**
- `crates/ralph-core/src/lib.rs` (add module)

**Implementation guidance:**
1. Create `Event` struct with `topic`, `payload`, `ts` fields
2. Create `EventReader` struct that tracks file position
3. Implement `read_new_events()` that returns events since last read
4. Handle edge cases: file doesn't exist, empty file, corrupt JSON
5. Use `serde_json` for parsing

**Test requirements:**
- Read events from valid JSONL file
- Handle empty file gracefully
- Skip invalid JSON lines with warning
- Track position correctly across multiple reads

**Demo:** Unit tests pass, can read events from sample JSONL file.

---

## Step 3: Create `HatlessRalph` struct and prompt builder

**Objective:** Implement the constant coordinator that's always present.

**Files to create:**
- `crates/ralph-core/src/hatless_ralph.rs`

**Files to modify:**
- `crates/ralph-core/src/lib.rs` (add module)

**Implementation guidance:**
1. Create `HatlessRalph` struct with `backend` and `hat_topology`
2. Implement `build_prompt()` with core prompt + conditional sections
3. Implement `should_handle()` to check if Ralph handles an event
4. Create `HatTopology` struct for injecting available hats into prompt

**Test requirements:**
- Solo mode prompt (no hats) includes SOLO MODE section
- Multi-hat prompt includes MY TEAM section with hat table
- Prompt always includes core behaviors (scratchpad, backpressure)

**Demo:** `HatlessRalph::build_prompt()` generates correct prompts for solo and multi-hat modes.

---

## Step 4: Modify `HatRegistry` to remove default hats

**Objective:** Registry holds only user-defined hats; Ralph is separate.

**Files to modify:**
- `crates/ralph-core/src/hat_registry.rs`

**Implementation guidance:**
1. Remove `default_planner()` and `default_builder()` auto-creation
2. `from_config()` only registers hats from config, no defaults
3. Add `has_subscriber(&Topic)` method
4. Add `get_for_topic(&Topic)` method
5. Add `to_topology()` for prompt injection

**Test requirements:**
- Empty config results in empty registry (not default planner/builder)
- `has_subscriber()` returns false for unsubscribed topics
- `get_for_topic()` returns correct hat for matching trigger

**Demo:** `HatRegistry::from_config(&empty_config)` returns empty registry.

---

## Step 5: Update `InstructionBuilder` with `build_hatless_ralph()`

**Objective:** New prompt building method for hatless Ralph.

**Files to modify:**
- `crates/ralph-core/src/instructions.rs`

**Implementation guidance:**
1. Add `build_hatless_ralph(&Context)` method
2. Define `RALPH_CORE_PROMPT` constant with identity and core behaviors
3. Define `SOLO_MODE_SECTION` constant
4. Define `MULTI_HAT_SECTION_TEMPLATE` for hat topology injection
5. Include JSONL event writing instructions in prompt

**Test requirements:**
- Core prompt includes scratchpad, backpressure, event writing instructions
- Solo mode section only appears when no hats configured
- Multi-hat section includes correct topology table

**Demo:** Generated prompts match expected format from design doc.

---

## Step 6: Modify `EventLoop` to use Ralph as fallback

**Objective:** Ralph handles events with no subscriber; owns completion.

**Files to modify:**
- `crates/ralph-core/src/event_loop.rs`

**Implementation guidance:**
1. Add `ralph: HatlessRalph` field to `EventLoop`
2. Modify `next_hat()` to return `None` for unsubscribed events
3. When no hat found, invoke Ralph instead
4. Only accept `LOOP_COMPLETE` from Ralph iterations
5. Replace XML event parsing with `EventReader`

**Test requirements:**
- Unsubscribed event triggers Ralph
- `LOOP_COMPLETE` from hat is ignored
- `LOOP_COMPLETE` from Ralph terminates loop
- Events read from JSONL, not parsed from output

**Demo:** Integration test shows orphaned event falling through to Ralph.

---

## Step 7: Implement `default_publishes` fallback logic

**Objective:** Hats that forget to write events use their default.

**Files to modify:**
- `crates/ralph-core/src/event_loop.rs`

**Implementation guidance:**
1. After hat execution, check if new events were written
2. If no events and hat has `default_publishes`, inject default event
3. Write default event to `.agent/events.jsonl`
4. Log when default event is used

**Test requirements:**
- Hat writes event → event is used
- Hat writes no event + has default → default is used
- Hat writes no event + no default → falls through to Ralph

**Demo:** Test scenario where hat forgets event, default fires.

---

## Step 8: Add per-hat backend resolution

**Objective:** Each hat uses its configured backend.

**Files to modify:**
- `crates/ralph-core/src/event_loop.rs`
- `crates/ralph-adapters/src/cli_backend.rs`

**Implementation guidance:**
1. When selecting executor for hat, resolve its `HatBackend`
2. If hat has no backend, inherit from `cli.backend`
3. Add `kiro_with_agent()` constructor to `CliBackend`
4. Add `from_name()` constructor for named backends

**Test requirements:**
- Hat with `backend: "claude"` uses Claude backend
- Hat with `backend: { type: "kiro", agent: "x" }` uses Kiro with --agent flag
- Hat without backend inherits global default

**Demo:** Mixed backend config executes correct commands per hat.

---

## Step 9: Update presets (remove planner hat)

**Objective:** Presets define hats only; Ralph is implicit.

**Files to modify:**
- `presets/feature.yml`
- `presets/feature-minimal.yml`
- `presets/review.yml`
- `presets/research.yml`
- `presets/debug.yml`
- `presets/docs.yml`
- `presets/refactor.yml`
- `presets/deploy.yml`
- `presets/gap-analysis.yml`

**Implementation guidance:**
1. Remove `planner` hat from all presets
2. Ensure entry events route to Ralph or appropriate hat
3. Add `default_publishes` where appropriate
4. Update instructions to write JSONL events
5. Test each preset with `ralph validate`

**Test requirements:**
- All presets pass validation
- No preset has `planner` hat
- All presets have valid event flow (no orphans)

**Demo:** `ralph validate presets/*.yml` all pass.

---

## Step 10: Create mock CLI test harness

**Objective:** Deterministic E2E testing without real CLI tools.

**Files to create:**
- `crates/ralph-core/src/testing/mod.rs`
- `crates/ralph-core/src/testing/mock_backend.rs`
- `crates/ralph-core/src/testing/scenario.rs`

**Implementation guidance:**
1. Create `MockCliBackend` that reads scripted responses
2. Create `Scenario` struct parsed from YAML
3. Implement scenario loading and execution
4. Mock backend writes scripted files to temp directory
5. Add assertions for completion, iteration count, file state

**Test requirements:**
- Scenarios load from YAML
- Mock backend executes scripted iterations
- Assertions verify expected outcomes

**Demo:** Sample scenario runs successfully with mock backend.

---

## Step 11: Write E2E scenario tests

**Objective:** Comprehensive test coverage via scenarios.

**Files to create:**
- `tests/scenarios/solo_mode_complete.yml`
- `tests/scenarios/multi_hat_delegation.yml`
- `tests/scenarios/orphaned_event_fallback.yml`
- `tests/scenarios/default_publishes.yml`
- `tests/scenarios/hat_to_hat_direct.yml`
- `tests/scenarios/mixed_backends.yml`
- `tests/scenarios/completion_only_from_ralph.yml`

**Implementation guidance:**
1. Write scenario YAML for each key behavior
2. Create test runner that executes all scenarios
3. Add to CI pipeline

**Test requirements:**
- All scenarios in checklist are covered
- Tests run in CI
- Clear failure messages when scenarios fail

**Demo:** `cargo test --test scenarios` passes all scenarios.

---

## Step 12: Update documentation and migration guide

**Objective:** Users can migrate to new architecture smoothly.

**Files to modify:**
- `docs/guide/configuration.md`
- `docs/guide/agents.md`
- `docs/changelog.md`

**Files to create:**
- `docs/migration/v2-hatless-ralph.md`

**Implementation guidance:**
1. Document new config schema (per-hat backends, default_publishes)
2. Document JSONL event format
3. Write migration guide for existing users
4. Update CHANGELOG with breaking changes
5. Add examples for common configurations

**Test requirements:**
- Documentation builds without errors
- Examples in docs are valid YAML

**Demo:** Migration guide enables user to update existing config.

---

## Implementation Order Rationale

The steps are ordered to build foundational components first:

1. **Steps 1-2:** Data structures and parsing (no behavioral changes yet)
2. **Steps 3-5:** New components (HatlessRalph, updated registry, prompts)
3. **Steps 6-8:** Core behavioral changes (event loop, fallback, backends)
4. **Steps 9:** Preset updates (depends on new architecture working)
5. **Steps 10-11:** Testing infrastructure (validates everything works)
6. **Step 12:** Documentation (after implementation is stable)

Each step results in working, testable code that builds on previous steps.
