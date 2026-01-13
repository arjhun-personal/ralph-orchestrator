//! Event loop orchestration.
//!
//! The event loop coordinates the execution of hats via pub/sub messaging.

use crate::config::RalphConfig;
use crate::event_parser::EventParser;
use crate::hat_registry::HatRegistry;
use crate::instructions::InstructionBuilder;
use ralph_proto::{Event, EventBus, HatId};
use std::time::{Duration, Instant};

/// Reason the event loop terminated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminationReason {
    /// Completion promise was detected in output.
    CompletionPromise,
    /// Maximum iterations reached.
    MaxIterations,
    /// Maximum runtime exceeded.
    MaxRuntime,
    /// Maximum cost exceeded.
    MaxCost,
    /// Too many consecutive failures.
    ConsecutiveFailures,
    /// Manually stopped.
    Stopped,
}

/// Current state of the event loop.
#[derive(Debug)]
pub struct LoopState {
    /// Current iteration number (1-indexed).
    pub iteration: u32,
    /// Number of consecutive failures.
    pub consecutive_failures: u32,
    /// Cumulative cost in USD (if tracked).
    pub cumulative_cost: f64,
    /// When the loop started.
    pub started_at: Instant,
    /// The last hat that executed.
    pub last_hat: Option<HatId>,
    /// Number of git checkpoints created.
    pub checkpoint_count: u32,
}

impl Default for LoopState {
    fn default() -> Self {
        Self {
            iteration: 0,
            consecutive_failures: 0,
            cumulative_cost: 0.0,
            started_at: Instant::now(),
            last_hat: None,
            checkpoint_count: 0,
        }
    }
}

impl LoopState {
    /// Creates a new loop state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the elapsed time since the loop started.
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }
}

/// The main event loop orchestrator.
pub struct EventLoop {
    config: RalphConfig,
    registry: HatRegistry,
    bus: EventBus,
    state: LoopState,
    instruction_builder: InstructionBuilder,
}

impl EventLoop {
    /// Creates a new event loop from configuration.
    pub fn new(config: RalphConfig) -> Self {
        let registry = HatRegistry::from_config(&config);
        let instruction_builder = InstructionBuilder::new(&config.event_loop.completion_promise);

        let mut bus = EventBus::new();
        for hat in registry.all() {
            bus.register(hat.clone());
        }

        Self {
            config,
            registry,
            bus,
            state: LoopState::new(),
            instruction_builder,
        }
    }

    /// Returns the current loop state.
    pub fn state(&self) -> &LoopState {
        &self.state
    }

    /// Returns the configuration.
    pub fn config(&self) -> &RalphConfig {
        &self.config
    }

    /// Returns the hat registry.
    pub fn registry(&self) -> &HatRegistry {
        &self.registry
    }

    /// Checks if any termination condition is met.
    pub fn check_termination(&self) -> Option<TerminationReason> {
        let cfg = &self.config.event_loop;

        if self.state.iteration >= cfg.max_iterations {
            return Some(TerminationReason::MaxIterations);
        }

        if self.state.elapsed().as_secs() >= cfg.max_runtime_seconds {
            return Some(TerminationReason::MaxRuntime);
        }

        if let Some(max_cost) = cfg.max_cost_usd {
            if self.state.cumulative_cost >= max_cost {
                return Some(TerminationReason::MaxCost);
            }
        }

        if self.state.consecutive_failures >= cfg.max_consecutive_failures {
            return Some(TerminationReason::ConsecutiveFailures);
        }

        None
    }

    /// Initializes the loop by publishing the start event.
    pub fn initialize(&mut self, prompt_content: &str) {
        let start_event = Event::new("task.start", prompt_content);
        self.bus.publish(start_event);
    }

    /// Gets the next hat to execute (if any have pending events).
    pub fn next_hat(&self) -> Option<&HatId> {
        self.bus.next_hat_with_pending()
    }

    /// Builds the prompt for a hat's execution.
    pub fn build_prompt(&mut self, hat_id: &HatId) -> Option<String> {
        let hat = self.registry.get(hat_id)?;

        if self.config.is_single_mode() {
            // For single-hat mode, we need the prompt file content
            // This will be passed in by the caller
            None
        } else {
            // Multi-hat mode: build from pending events
            let events = self.bus.take_pending(&hat_id.clone());
            let events_context = events
                .iter()
                .map(|e| format!("Event: {} - {}", e.topic, e.payload))
                .collect::<Vec<_>>()
                .join("\n");
            Some(self.instruction_builder.build_multi_hat(hat, &events_context))
        }
    }

    /// Builds the single-hat mode prompt with given content.
    pub fn build_single_prompt(&self, prompt_content: &str) -> String {
        self.instruction_builder.build_single_hat(prompt_content)
    }

    /// Processes output from a hat execution.
    ///
    /// Returns the termination reason if the loop should stop.
    pub fn process_output(
        &mut self,
        hat_id: &HatId,
        output: &str,
        success: bool,
    ) -> Option<TerminationReason> {
        self.state.iteration += 1;
        self.state.last_hat = Some(hat_id.clone());

        // Track failures
        if success {
            self.state.consecutive_failures = 0;
        } else {
            self.state.consecutive_failures += 1;
        }

        // Check for completion promise
        if EventParser::contains_promise(output, &self.config.event_loop.completion_promise) {
            return Some(TerminationReason::CompletionPromise);
        }

        // Parse and publish events from output
        let parser = EventParser::new().with_source(hat_id.clone());
        let events = parser.parse(output);

        for event in events {
            self.bus.publish(event);
        }

        // If single-hat mode and no completion, publish continue event
        if self.config.is_single_mode() && self.bus.next_hat_with_pending().is_none() {
            let continue_event = Event::new("task.continue", "Continue with the task");
            self.bus.publish(continue_event);
        }

        // Check termination conditions
        self.check_termination()
    }

    /// Returns true if a checkpoint should be created at this iteration.
    pub fn should_checkpoint(&self) -> bool {
        let interval = self.config.event_loop.checkpoint_interval;
        interval > 0 && self.state.iteration % interval == 0
    }

    /// Adds cost to the cumulative total.
    pub fn add_cost(&mut self, cost: f64) {
        self.state.cumulative_cost += cost;
    }

    /// Records that a checkpoint was created.
    pub fn record_checkpoint(&mut self) {
        self.state.checkpoint_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_mode_initialization() {
        let config = RalphConfig::default();
        let mut event_loop = EventLoop::new(config);

        event_loop.initialize("Test prompt");

        // Default hat should have pending event
        assert!(event_loop.next_hat().is_some());
    }

    #[test]
    fn test_termination_max_iterations() {
        let yaml = r#"
event_loop:
  max_iterations: 2
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        let mut event_loop = EventLoop::new(config);
        event_loop.state.iteration = 2;

        assert_eq!(
            event_loop.check_termination(),
            Some(TerminationReason::MaxIterations)
        );
    }

    #[test]
    fn test_completion_promise_detection() {
        let config = RalphConfig::default();
        let mut event_loop = EventLoop::new(config);
        event_loop.initialize("Test");

        let hat_id = HatId::new("default");
        let reason = event_loop.process_output(&hat_id, "Done! LOOP_COMPLETE", true);

        assert_eq!(reason, Some(TerminationReason::CompletionPromise));
    }

    #[test]
    fn test_checkpoint_interval() {
        let yaml = r#"
event_loop:
  checkpoint_interval: 5
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        let mut event_loop = EventLoop::new(config);

        event_loop.state.iteration = 4;
        assert!(!event_loop.should_checkpoint());

        event_loop.state.iteration = 5;
        assert!(event_loop.should_checkpoint());

        event_loop.state.iteration = 10;
        assert!(event_loop.should_checkpoint());
    }
}
