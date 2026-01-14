//! Test scenario definitions and execution.

use crate::config::RalphConfig;
use crate::event_loop::EventLoop;
use crate::event_reader::Event;
use super::mock_backend::MockBackend;

/// A test scenario definition.
#[derive(Debug)]
pub struct Scenario {
    pub name: String,
    pub config: RalphConfig,
    pub expected_events: Vec<Event>,
    pub expected_iterations: usize,
}

impl Scenario {
    /// Creates a new scenario.
    pub fn new(name: impl Into<String>, config: RalphConfig) -> Self {
        Self {
            name: name.into(),
            config,
            expected_events: Vec::new(),
            expected_iterations: 0,
        }
    }

    /// Sets expected events.
    pub fn with_events(mut self, events: Vec<Event>) -> Self {
        self.expected_events = events;
        self
    }

    /// Sets expected iteration count.
    pub fn with_iterations(mut self, count: usize) -> Self {
        self.expected_iterations = count;
        self
    }
}

/// Executes test scenarios with mock backend.
pub struct ScenarioRunner {
    backend: MockBackend,
}

impl ScenarioRunner {
    /// Creates a new scenario runner with mock backend.
    pub fn new(backend: MockBackend) -> Self {
        Self { backend }
    }

    /// Executes a scenario and returns the trace.
    pub fn run(&self, scenario: &Scenario) -> ExecutionTrace {
        let mut event_loop = EventLoop::new(scenario.config.clone());
        let prompt = scenario.config.prompt_file.as_deref().unwrap_or("");
        event_loop.initialize(prompt);

        let mut iterations = 0;
        let mut events = Vec::new();

        // Simulate iterations
        while iterations < scenario.expected_iterations {
            // In real execution, this would call the CLI backend
            // For now, just record the iteration
            iterations += 1;

            // Process any events from the mock backend
            if let Ok(has_events) = event_loop.process_events_from_jsonl() {
                if has_events {
                    // Events were processed
                }
            }
        }

        ExecutionTrace {
            iterations,
            events,
            final_state: event_loop.state().iteration,
        }
    }
}

/// Trace of a scenario execution.
#[derive(Debug)]
pub struct ExecutionTrace {
    pub iterations: usize,
    pub events: Vec<Event>,
    pub final_state: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RalphConfig;

    #[test]
    fn test_scenario_creation() {
        let config = RalphConfig::default();
        let scenario = Scenario::new("test", config)
            .with_iterations(3);

        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.expected_iterations, 3);
    }

    #[test]
    fn test_scenario_runner_executes() {
        let backend = MockBackend::new(vec!["ok".into()]);
        let runner = ScenarioRunner::new(backend);

        let config = RalphConfig::default();
        let scenario = Scenario::new("test", config).with_iterations(1);

        let trace = runner.run(&scenario);
        assert_eq!(trace.iterations, 1);
    }

    #[test]
    fn test_mock_backend_simulates_hat_execution() {
        // Demo: Simulate a hat execution with scripted response
        let responses = vec![
            r#"Building feature...
<event topic="build.done">
tests: pass
lint: pass
typecheck: pass
</event>"#.to_string(),
        ];

        let backend = MockBackend::new(responses);
        
        // Execute once
        let output = backend.execute("You are the builder hat. Build feature X.");
        
        // Verify response
        assert!(output.contains("build.done"));
        assert!(output.contains("tests: pass"));
        
        // Verify execution was tracked
        assert_eq!(backend.execution_count(), 1);
        let executions = backend.executions();
        assert_eq!(executions[0].prompt, "You are the builder hat. Build feature X.");
    }
}
