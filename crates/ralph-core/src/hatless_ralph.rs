//! Hatless Ralph - the constant coordinator.
//!
//! Ralph is always present, cannot be configured away, and acts as a universal fallback.

use crate::config::CoreConfig;
use crate::hat_registry::HatRegistry;
use ralph_proto::Topic;

/// Hatless Ralph - the constant coordinator.
pub struct HatlessRalph {
    completion_promise: String,
    core: CoreConfig,
    hat_topology: Option<HatTopology>,
    /// Event to publish after coordination to start the hat workflow.
    starting_event: Option<String>,
}

/// Hat topology for multi-hat mode prompt generation.
pub struct HatTopology {
    hats: Vec<HatInfo>,
}

/// Information about a hat for prompt generation.
pub struct HatInfo {
    pub name: String,
    pub subscribes_to: Vec<String>,
    pub publishes: Vec<String>,
}

impl HatTopology {
    /// Creates topology from registry.
    pub fn from_registry(registry: &HatRegistry) -> Self {
        let hats = registry
            .all()
            .map(|hat| HatInfo {
                name: hat.name.clone(),
                subscribes_to: hat.subscriptions.iter().map(|t| t.as_str().to_string()).collect(),
                publishes: hat.publishes.iter().map(|t| t.as_str().to_string()).collect(),
            })
            .collect();

        Self { hats }
    }
}

impl HatlessRalph {
    /// Creates a new HatlessRalph.
    ///
    /// # Arguments
    /// * `completion_promise` - String that signals loop completion
    /// * `core` - Core configuration (scratchpad, specs_dir, guardrails)
    /// * `registry` - Hat registry for topology generation
    /// * `starting_event` - Optional event to publish after coordination to start hat workflow
    pub fn new(
        completion_promise: impl Into<String>,
        core: CoreConfig,
        registry: &HatRegistry,
        starting_event: Option<String>,
    ) -> Self {
        let hat_topology = if registry.is_empty() {
            None
        } else {
            Some(HatTopology::from_registry(registry))
        };

        Self {
            completion_promise: completion_promise.into(),
            core,
            hat_topology,
            starting_event,
        }
    }

    /// Builds Ralph's prompt based on context.
    pub fn build_prompt(&self, _context: &str) -> String {
        let mut prompt = self.core_prompt();
        prompt.push_str(&self.workflow_section());

        if let Some(topology) = &self.hat_topology {
            prompt.push_str(&self.hats_section(topology));
        }

        prompt.push_str(&self.event_writing_section());
        prompt.push_str(&self.done_section());

        prompt
    }

    /// Always returns true - Ralph handles all events as fallback.
    pub fn should_handle(&self, _topic: &Topic) -> bool {
        true
    }

    fn core_prompt(&self) -> String {
        let guardrails = self
            .core
            .guardrails
            .iter()
            .enumerate()
            .map(|(i, g)| format!("{}. {g}", 999 + i))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r"I'm Ralph. Fresh context each iteration.

### 0a. ORIENTATION
Study `{specs_dir}` to understand requirements.
Don't assume features aren't implemented—search first.

### 0b. SCRATCHPAD
Study `{scratchpad}`. It's shared state. It's memory.

Task markers:
- `[ ]` pending
- `[x]` done
- `[~]` cancelled (with reason)

### GUARDRAILS
{guardrails}

",
            scratchpad = self.core.scratchpad,
            specs_dir = self.core.specs_dir,
            guardrails = guardrails,
        )
    }

    fn workflow_section(&self) -> String {
        // Different workflow for solo mode vs multi-hat mode
        if self.hat_topology.is_some() {
            // Multi-hat mode: Ralph coordinates and delegates
            format!(
                r"## WORKFLOW

### 1. GAP ANALYSIS
Compare specs against codebase. Use parallel subagents (up to 10) for searches.

### 2. PLAN
Update `{scratchpad}` with prioritized tasks.

### 3. DELEGATE
Publish the starting event to hand off to specialized hats.
**DO NOT implement yourself** — that's what the hats are for.

",
                scratchpad = self.core.scratchpad
            )
        } else {
            // Solo mode: Ralph does everything
            format!(
                r"## WORKFLOW

### 1. GAP ANALYSIS
Compare specs against codebase. Use parallel subagents (up to 10) for searches.

### 2. PLAN
Update `{scratchpad}` with prioritized tasks.

### 3. IMPLEMENT
Pick ONE task. Only 1 subagent for build/tests.

### 4. COMMIT
Capture the why, not just the what. Mark `[x]` in scratchpad.

### 5. REPEAT
Until all tasks `[x]` or `[~]`.

",
                scratchpad = self.core.scratchpad
            )
        }
    }

    fn hats_section(&self, topology: &HatTopology) -> String {
        let mut section = String::from("## HATS\n\nDelegate via events.\n\n");

        // Include starting_event instruction if configured
        if let Some(ref starting_event) = self.starting_event {
            section.push_str(&format!(
                "**After coordination, publish `{}` to start the workflow.**\n\n",
                starting_event
            ));
        }

        // Build hat table
        section.push_str("| Hat | Triggers On | Publishes |\n");
        section.push_str("|-----|-------------|----------|\n");

        for hat in &topology.hats {
            let subscribes = hat.subscribes_to.join(", ");
            let publishes = hat.publishes.join(", ");
            section.push_str(&format!("| {} | {} | {} |\n", hat.name, subscribes, publishes));
        }

        section.push('\n');
        section
    }

    fn event_writing_section(&self) -> String {
        format!(
            r#"## EVENT WRITING

Write events to `{events_file}` as:
{{"topic": "build.task", "payload": "...", "ts": "2026-01-14T12:00:00Z"}}

"#,
            events_file = ".agent/events.jsonl"
        )
    }

    fn done_section(&self) -> String {
        format!(
            r"## DONE

Output {} when all tasks complete.
",
            self.completion_promise
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RalphConfig;

    #[test]
    fn test_prompt_without_hats() {
        let config = RalphConfig::default();
        let registry = HatRegistry::new(); // Empty registry
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let prompt = ralph.build_prompt("");

        // Identity with ghuntley style
        assert!(prompt.contains("I'm Ralph. Fresh context each iteration."));

        // Numbered orientation phases
        assert!(prompt.contains("### 0a. ORIENTATION"));
        assert!(prompt.contains("Study"));
        assert!(prompt.contains("Don't assume features aren't implemented"));

        // Scratchpad section with task markers
        assert!(prompt.contains("### 0b. SCRATCHPAD"));
        assert!(prompt.contains("Task markers:"));
        assert!(prompt.contains("- `[ ]` pending"));
        assert!(prompt.contains("- `[x]` done"));
        assert!(prompt.contains("- `[~]` cancelled"));

        // Workflow with numbered steps
        assert!(prompt.contains("## WORKFLOW"));
        assert!(prompt.contains("### 1. GAP ANALYSIS"));
        assert!(prompt.contains("Use parallel subagents (up to 10)"));
        assert!(prompt.contains("### 2. PLAN"));
        assert!(prompt.contains("### 3. IMPLEMENT"));
        assert!(prompt.contains("Only 1 subagent for build/tests"));
        assert!(prompt.contains("### 4. COMMIT"));
        assert!(prompt.contains("Capture the why"));
        assert!(prompt.contains("### 5. REPEAT"));

        // Should NOT have hats section when no hats
        assert!(!prompt.contains("## HATS"));

        // Event writing and completion
        assert!(prompt.contains("## EVENT WRITING"));
        assert!(prompt.contains(".agent/events.jsonl"));
        assert!(prompt.contains("LOOP_COMPLETE"));
    }

    #[test]
    fn test_prompt_with_hats() {
        // Note: using semantic events since task.start is reserved for Ralph
        let yaml = r#"
hats:
  planner:
    name: "Planner"
    triggers: ["planning.start", "build.done", "build.blocked"]
    publishes: ["build.task"]
  builder:
    name: "Builder"
    triggers: ["build.task"]
    publishes: ["build.done", "build.blocked"]
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        let registry = HatRegistry::from_config(&config);
        let ralph = HatlessRalph::new(
            "LOOP_COMPLETE",
            config.core.clone(),
            &registry,
            Some("planning.start".to_string()),
        );

        let prompt = ralph.build_prompt("");

        // Identity with ghuntley style
        assert!(prompt.contains("I'm Ralph. Fresh context each iteration."));

        // Orientation phases
        assert!(prompt.contains("### 0a. ORIENTATION"));
        assert!(prompt.contains("### 0b. SCRATCHPAD"));

        // Multi-hat workflow: DELEGATE, not IMPLEMENT
        assert!(prompt.contains("## WORKFLOW"));
        assert!(prompt.contains("### 1. GAP ANALYSIS"));
        assert!(prompt.contains("### 3. DELEGATE"), "Multi-hat mode should have DELEGATE step");
        assert!(
            !prompt.contains("### 3. IMPLEMENT"),
            "Multi-hat mode should NOT tell Ralph to implement"
        );
        assert!(
            prompt.contains("DO NOT implement yourself"),
            "Should explicitly tell Ralph not to implement"
        );

        // Hats section when hats are defined
        assert!(prompt.contains("## HATS"));
        assert!(prompt.contains("Delegate via events"));
        assert!(prompt.contains("| Hat | Triggers On | Publishes |"));

        // Event writing and completion
        assert!(prompt.contains("## EVENT WRITING"));
        assert!(prompt.contains("LOOP_COMPLETE"));
    }

    #[test]
    fn test_should_handle_always_true() {
        let config = RalphConfig::default();
        let registry = HatRegistry::new();
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        assert!(ralph.should_handle(&Topic::new("any.topic")));
        assert!(ralph.should_handle(&Topic::new("build.task")));
        assert!(ralph.should_handle(&Topic::new("unknown.event")));
    }

    #[test]
    fn test_ghuntley_patterns_present() {
        let config = RalphConfig::default();
        let registry = HatRegistry::new();
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let prompt = ralph.build_prompt("");

        // Key ghuntley language patterns
        assert!(prompt.contains("Study"), "Should use 'study' verb");
        assert!(
            prompt.contains("Don't assume features aren't implemented"),
            "Should have 'don't assume' guardrail"
        );
        assert!(
            prompt.contains("parallel subagents"),
            "Should mention parallel subagents for reads"
        );
        assert!(
            prompt.contains("Only 1 subagent"),
            "Should limit to 1 subagent for builds"
        );
        assert!(
            prompt.contains("Capture the why"),
            "Should emphasize 'why' in commits"
        );

        // Numbered guardrails (999+)
        assert!(prompt.contains("### GUARDRAILS"), "Should have guardrails section");
        assert!(prompt.contains("999."), "Guardrails should use high numbers");
    }

    #[test]
    fn test_scratchpad_format_documented() {
        let config = RalphConfig::default();
        let registry = HatRegistry::new();
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let prompt = ralph.build_prompt("");

        // Task marker format is documented
        assert!(prompt.contains("- `[ ]` pending"));
        assert!(prompt.contains("- `[x]` done"));
        assert!(prompt.contains("- `[~]` cancelled (with reason)"));
    }

    #[test]
    fn test_starting_event_in_prompt() {
        // When starting_event is configured, prompt should include delegation instruction
        let yaml = r#"
hats:
  tdd_writer:
    name: "TDD Writer"
    triggers: ["tdd.start"]
    publishes: ["test.written"]
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        let registry = HatRegistry::from_config(&config);
        let ralph = HatlessRalph::new(
            "LOOP_COMPLETE",
            config.core.clone(),
            &registry,
            Some("tdd.start".to_string()),
        );

        let prompt = ralph.build_prompt("");

        // Should include delegation instruction
        assert!(
            prompt.contains("After coordination, publish `tdd.start` to start the workflow"),
            "Prompt should include starting_event delegation instruction"
        );
    }

    #[test]
    fn test_no_starting_event_instruction_when_none() {
        // When starting_event is None, no delegation instruction should appear
        let yaml = r#"
hats:
  some_hat:
    name: "Some Hat"
    triggers: ["some.event"]
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        let registry = HatRegistry::from_config(&config);
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let prompt = ralph.build_prompt("");

        // Should NOT include delegation instruction
        assert!(
            !prompt.contains("After coordination, publish"),
            "Prompt should NOT include starting_event delegation when None"
        );
    }
}
