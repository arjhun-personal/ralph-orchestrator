//! Chaos Mode Runner
//!
//! Chaos mode activates ONLY after LOOP_COMPLETE to grow the original
//! objective into related improvements and learnings.
//!
//! Key design:
//! - Uses the original objective as a "seed" to explore related improvements
//! - Has configurable research focus areas and outputs
//! - Includes cooldown between iterations
//! - Outputs only memories by default (safe mode)

use crate::config::{ChaosModeConfig, ChaosOutput, ResearchFocus};
use std::time::{Duration, Instant};

/// State for chaos mode execution.
#[derive(Debug, Clone)]
pub struct ChaosModeState {
    /// Original objective that completed successfully.
    pub original_objective: String,
    /// Current chaos mode iteration (0-indexed).
    pub iteration: u32,
    /// Maximum iterations allowed.
    pub max_iterations: u32,
    /// When the current cooldown started.
    pub cooldown_started: Option<Instant>,
    /// Cooldown duration between iterations.
    pub cooldown_duration: Duration,
    /// Research focus areas for this run.
    pub research_focus: Vec<ResearchFocus>,
    /// Allowed outputs for this run.
    pub outputs: Vec<ChaosOutput>,
}

impl ChaosModeState {
    /// Creates new chaos mode state from config.
    pub fn new(original_objective: impl Into<String>, config: &ChaosModeConfig) -> Self {
        Self {
            original_objective: original_objective.into(),
            iteration: 0,
            max_iterations: config.max_iterations,
            cooldown_started: None,
            cooldown_duration: Duration::from_secs(config.cooldown_seconds),
            research_focus: config.research_focus.clone(),
            outputs: config.outputs.clone(),
        }
    }

    /// Returns true if chaos mode should continue.
    pub fn should_continue(&self) -> bool {
        self.iteration < self.max_iterations
    }

    /// Returns true if cooldown has elapsed.
    pub fn cooldown_elapsed(&self) -> bool {
        match self.cooldown_started {
            Some(started) => started.elapsed() >= self.cooldown_duration,
            None => true, // No cooldown started means we can proceed
        }
    }

    /// Starts cooldown for next iteration.
    pub fn start_cooldown(&mut self) {
        self.cooldown_started = Some(Instant::now());
    }

    /// Advances to next iteration.
    pub fn next_iteration(&mut self) {
        self.iteration += 1;
        self.cooldown_started = None;
    }

    /// Returns remaining cooldown time, if any.
    pub fn remaining_cooldown(&self) -> Option<Duration> {
        self.cooldown_started.and_then(|started| {
            let elapsed = started.elapsed();
            self.cooldown_duration.checked_sub(elapsed)
        })
    }

    /// Returns true if memories output is enabled.
    pub fn can_output_memories(&self) -> bool {
        self.outputs.contains(&ChaosOutput::Memories)
    }

    /// Returns true if tasks output is enabled.
    pub fn can_output_tasks(&self) -> bool {
        self.outputs.contains(&ChaosOutput::Tasks)
    }

    /// Returns true if specs output is enabled.
    pub fn can_output_specs(&self) -> bool {
        self.outputs.contains(&ChaosOutput::Specs)
    }

    /// Builds the chaos mode prompt section for HatlessRalph.
    ///
    /// This generates the workflow instructions for chaos mode iterations.
    pub fn build_prompt_section(&self) -> String {
        let mut prompt = String::new();

        prompt.push_str("## CHAOS MODE\n\n");
        prompt.push_str(&format!(
            "**Iteration {}/{}** - Exploring related improvements.\n\n",
            self.iteration + 1,
            self.max_iterations
        ));

        prompt.push_str("**Original objective (seed):**\n> ");
        prompt.push_str(&self.original_objective);
        prompt.push_str("\n\n");

        // Research focus areas
        prompt.push_str("### Research Focus\n\n");
        prompt.push_str("Explore improvements in these areas:\n");
        for focus in &self.research_focus {
            let (name, desc) = match focus {
                ResearchFocus::DomainBestPractices => (
                    "Domain Best Practices",
                    "Research industry patterns and best practices for similar implementations",
                ),
                ResearchFocus::CodebasePatterns => (
                    "Codebase Patterns",
                    "Analyze existing code for patterns, conventions, and potential improvements",
                ),
                ResearchFocus::SelfImprovement => (
                    "Self Improvement",
                    "Study how the orchestration could work better (meta-level improvements)",
                ),
            };
            prompt.push_str(&format!("- **{}**: {}\n", name, desc));
        }
        prompt.push('\n');

        // Allowed outputs
        prompt.push_str("### Allowed Outputs\n\n");
        if self.can_output_memories() {
            prompt.push_str("- **Memories** ✓ - Capture learnings in `.ralph/agent/memories.md`\n");
        }
        if self.can_output_tasks() {
            prompt.push_str(
                "- **Tasks** ✓ - Create tasks for concrete improvements via `ralph tools task add`\n",
            );
        }
        if self.can_output_specs() {
            prompt.push_str(
                "- **Specs** ✓ - Draft specs for larger improvements in `.ralph/specs/`\n",
            );
        }
        prompt.push('\n');

        // Workflow
        prompt.push_str("### Chaos Workflow\n\n");
        prompt.push_str("1. **STUDY** - Analyze the original objective and how it was completed\n");
        prompt.push_str("2. **RESEARCH** - Explore one focus area for improvements\n");
        prompt.push_str("3. **CAPTURE** - Record one concrete insight or improvement\n");
        prompt.push_str(
            "4. **EXIT** - Output `CHAOS_COMPLETE` when satisfied, or continue exploring\n\n",
        );

        prompt.push_str("**CRITICAL:** Each iteration should capture exactly ONE insight.\n");
        prompt.push_str("Quality over quantity - prefer deep exploration over surface scanning.\n");

        prompt
    }
}

/// Chaos mode completion promise.
pub const CHAOS_COMPLETION_PROMISE: &str = "CHAOS_COMPLETE";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_state_defaults() {
        let config = ChaosModeConfig::default();
        let state = ChaosModeState::new("Fix the login bug", &config);

        assert_eq!(state.original_objective, "Fix the login bug");
        assert_eq!(state.iteration, 0);
        assert_eq!(state.max_iterations, 5);
        assert!(state.should_continue());
        assert!(state.cooldown_elapsed());
        assert!(state.can_output_memories());
        assert!(!state.can_output_tasks());
        assert!(!state.can_output_specs());
    }

    #[test]
    fn test_iteration_advancement() {
        let config = ChaosModeConfig::default();
        let mut state = ChaosModeState::new("Build feature X", &config);

        assert_eq!(state.iteration, 0);
        state.next_iteration();
        assert_eq!(state.iteration, 1);
        assert!(state.should_continue());

        // Advance to max
        for _ in 0..4 {
            state.next_iteration();
        }
        assert_eq!(state.iteration, 5);
        assert!(!state.should_continue());
    }

    #[test]
    fn test_cooldown_tracking() {
        let mut config = ChaosModeConfig::default();
        config.cooldown_seconds = 1; // 1 second for testing
        let mut state = ChaosModeState::new("Test", &config);

        assert!(state.cooldown_elapsed()); // No cooldown yet
        assert!(state.remaining_cooldown().is_none());

        state.start_cooldown();
        assert!(!state.cooldown_elapsed()); // Cooldown just started
        assert!(state.remaining_cooldown().is_some());
    }

    #[test]
    fn test_prompt_section_generation() {
        let config = ChaosModeConfig::default();
        let state = ChaosModeState::new("Implement user auth", &config);

        let prompt = state.build_prompt_section();

        assert!(prompt.contains("## CHAOS MODE"));
        assert!(prompt.contains("Iteration 1/5"));
        assert!(prompt.contains("Implement user auth"));
        assert!(prompt.contains("### Research Focus"));
        assert!(prompt.contains("Domain Best Practices"));
        assert!(prompt.contains("### Allowed Outputs"));
        assert!(prompt.contains("Memories"));
        assert!(prompt.contains("### Chaos Workflow"));
        assert!(prompt.contains("CHAOS_COMPLETE"));
    }

    #[test]
    fn test_custom_outputs() {
        let mut config = ChaosModeConfig::default();
        config.outputs = vec![
            ChaosOutput::Memories,
            ChaosOutput::Tasks,
            ChaosOutput::Specs,
        ];
        let state = ChaosModeState::new("Test", &config);

        assert!(state.can_output_memories());
        assert!(state.can_output_tasks());
        assert!(state.can_output_specs());
    }
}
