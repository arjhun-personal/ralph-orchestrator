//! Tier 3: Event System test scenarios.
//!
//! These scenarios test Ralph's event handling capabilities:
//! - Event XML parsing from agent output
//! - Backpressure verification (build.done evidence)
//!
//! Events are the primary communication mechanism between the agent and Ralph,
//! so reliable parsing is critical for orchestration correctness.

use super::{Assertions, ScenarioError, TestScenario};
use crate::Backend;
use crate::executor::{PromptSource, RalphExecutor, ScenarioConfig};
use crate::models::TestResult;
use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;

/// Test scenario that verifies event XML parsing.
///
/// This scenario:
/// - Sends a prompt that instructs the agent to emit specific events
/// - Verifies that Ralph correctly parses the `<event topic="...">` XML format
/// - Validates that event topics and payloads are captured accurately
///
/// # Example
///
/// ```no_run
/// use ralph_e2e::scenarios::{ClaudeEventsScenario, TestScenario};
///
/// let scenario = ClaudeEventsScenario::new();
/// assert_eq!(scenario.tier(), "Tier 3: Events");
/// ```
pub struct ClaudeEventsScenario {
    id: String,
    description: String,
    tier: String,
}

impl ClaudeEventsScenario {
    /// Creates a new event parsing scenario.
    pub fn new() -> Self {
        Self {
            id: "claude-events".to_string(),
            description: "Verifies event XML parsing from agent output".to_string(),
            tier: "Tier 3: Events".to_string(),
        }
    }
}

impl Default for ClaudeEventsScenario {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TestScenario for ClaudeEventsScenario {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn tier(&self) -> &str {
        &self.tier
    }

    fn backend(&self) -> Backend {
        Backend::Claude
    }

    fn setup(&self, workspace: &Path) -> Result<ScenarioConfig, ScenarioError> {
        // Create the .agent directory
        let agent_dir = workspace.join(".agent");
        std::fs::create_dir_all(&agent_dir).map_err(|e| {
            ScenarioError::SetupError(format!("failed to create .agent directory: {}", e))
        })?;

        // Create ralph.yml for event testing
        let config_content = r#"# Event parsing test config
cli:
  backend: claude

event_loop:
  max_iterations: 1
  completion_promise: "LOOP_COMPLETE"
"#;
        let config_path = workspace.join("ralph.yml");
        std::fs::write(&config_path, config_content)
            .map_err(|e| ScenarioError::SetupError(format!("failed to write ralph.yml: {}", e)))?;

        // Create a prompt that instructs the agent to emit specific events
        let prompt = r#"You are testing Ralph's event parsing system.

Your task is to emit a specific event using the XML format:
<event topic="test.event">Test payload data</event>

After emitting the event, output LOOP_COMPLETE to signal completion.

Do exactly this:
1. Output the event XML above
2. Output LOOP_COMPLETE

Nothing else."#;

        Ok(ScenarioConfig {
            config_file: "ralph.yml".into(),
            prompt: PromptSource::Inline(prompt.to_string()),
            max_iterations: 1,
            timeout: Duration::from_secs(300), // 5 minutes - backend iterations can be slow
            extra_args: vec![],
        })
    }

    async fn run(
        &self,
        executor: &RalphExecutor,
        config: &ScenarioConfig,
    ) -> Result<TestResult, ScenarioError> {
        let start = std::time::Instant::now();

        let execution = executor
            .run(config)
            .await
            .map_err(|e| ScenarioError::ExecutionError(format!("ralph execution failed: {}", e)))?;

        let duration = start.elapsed();

        // Build assertions for event parsing
        // Note: We use exit_code_success_or_limit() because Ralph's exit code 2 means
        // "max iterations reached" which is valid when functional behavior succeeds.
        let assertions = vec![
            Assertions::response_received(&execution),
            Assertions::exit_code_success_or_limit(&execution),
            Assertions::no_timeout(&execution),
            Assertions::event_emitted(&execution, "test.event"),
            self.event_has_payload(&execution, "test.event", "Test payload"),
        ];

        let all_passed = assertions.iter().all(|a| a.passed);

        Ok(TestResult {
            scenario_id: self.id.clone(),
            scenario_description: self.description.clone(),
            backend: self.backend().to_string(),
            tier: self.tier.clone(),
            passed: all_passed,
            assertions,
            duration,
        })
    }
}

impl ClaudeEventsScenario {
    /// Asserts that an event has the expected payload substring.
    fn event_has_payload(
        &self,
        result: &crate::executor::ExecutionResult,
        topic: &str,
        expected_substring: &str,
    ) -> crate::models::Assertion {
        let event = result.events.iter().find(|e| e.topic == topic);
        let has_payload = event
            .map(|e| e.payload.contains(expected_substring))
            .unwrap_or(false);

        super::AssertionBuilder::new(format!(
            "Event '{}' payload contains '{}'",
            topic, expected_substring
        ))
        .expected(format!("Payload containing '{}'", expected_substring))
        .actual(match event {
            Some(e) => format!("Payload: {}", truncate(&e.payload, 50)),
            None => "Event not found".to_string(),
        })
        .build()
        .with_passed(has_payload)
    }
}

/// Test scenario that verifies backpressure mechanism.
///
/// This scenario:
/// - Sends a prompt that requires passing tests before completion
/// - Verifies that `build.done` event provides evidence of backpressure checks
/// - Validates that the agent follows the backpressure protocol
///
/// Backpressure is Ralph's mechanism for ensuring code quality gates are passed
/// before work is considered complete.
///
/// # Example
///
/// ```no_run
/// use ralph_e2e::scenarios::{ClaudeBackpressureScenario, TestScenario};
///
/// let scenario = ClaudeBackpressureScenario::new();
/// assert_eq!(scenario.tier(), "Tier 3: Events");
/// ```
pub struct ClaudeBackpressureScenario {
    id: String,
    description: String,
    tier: String,
}

impl ClaudeBackpressureScenario {
    /// Creates a new backpressure scenario.
    pub fn new() -> Self {
        Self {
            id: "claude-backpressure".to_string(),
            description: "Verifies backpressure mechanism with build.done evidence".to_string(),
            tier: "Tier 3: Events".to_string(),
        }
    }
}

impl Default for ClaudeBackpressureScenario {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TestScenario for ClaudeBackpressureScenario {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn tier(&self) -> &str {
        &self.tier
    }

    fn backend(&self) -> Backend {
        Backend::Claude
    }

    fn setup(&self, workspace: &Path) -> Result<ScenarioConfig, ScenarioError> {
        // Create the .agent directory
        let agent_dir = workspace.join(".agent");
        std::fs::create_dir_all(&agent_dir).map_err(|e| {
            ScenarioError::SetupError(format!("failed to create .agent directory: {}", e))
        })?;

        // Create ralph.yml for backpressure testing
        let config_content = r#"# Backpressure test config
cli:
  backend: claude

event_loop:
  max_iterations: 2
  completion_promise: "LOOP_COMPLETE"
"#;
        let config_path = workspace.join("ralph.yml");
        std::fs::write(&config_path, config_content)
            .map_err(|e| ScenarioError::SetupError(format!("failed to write ralph.yml: {}", e)))?;

        // Create a prompt that exercises the backpressure protocol
        // NOTE: Must be explicit about including literal XML tags in output
        let prompt = r#"You are testing Ralph's backpressure mechanism.

Backpressure is Ralph's quality gate system. You must emit a build.done event.

IMPORTANT: You MUST include this EXACT XML in your response text:

<event topic="build.done">
tests: pass
lint: pass
</event>

After emitting the event above, output LOOP_COMPLETE on its own line.

Now emit the event:"#;

        Ok(ScenarioConfig {
            config_file: "ralph.yml".into(),
            prompt: PromptSource::Inline(prompt.to_string()),
            max_iterations: 2,
            timeout: Duration::from_secs(300), // 5 minutes - backend iterations can be slow
            extra_args: vec![],
        })
    }

    async fn run(
        &self,
        executor: &RalphExecutor,
        config: &ScenarioConfig,
    ) -> Result<TestResult, ScenarioError> {
        let start = std::time::Instant::now();

        let execution = executor
            .run(config)
            .await
            .map_err(|e| ScenarioError::ExecutionError(format!("ralph execution failed: {}", e)))?;

        let duration = start.elapsed();

        // Build assertions for backpressure verification
        // Note: We use exit_code_success_or_limit() because Ralph's exit code 2 means
        // "max iterations reached" which is valid when functional behavior succeeds.
        let assertions = vec![
            Assertions::response_received(&execution),
            Assertions::exit_code_success_or_limit(&execution),
            Assertions::no_timeout(&execution),
            Assertions::event_emitted(&execution, "build.done"),
            self.build_done_has_evidence(&execution),
        ];

        let all_passed = assertions.iter().all(|a| a.passed);

        Ok(TestResult {
            scenario_id: self.id.clone(),
            scenario_description: self.description.clone(),
            backend: self.backend().to_string(),
            tier: self.tier.clone(),
            passed: all_passed,
            assertions,
            duration,
        })
    }
}

impl ClaudeBackpressureScenario {
    /// Asserts that build.done event contains verification evidence.
    fn build_done_has_evidence(
        &self,
        result: &crate::executor::ExecutionResult,
    ) -> crate::models::Assertion {
        let event = result.events.iter().find(|e| e.topic == "build.done");
        let has_evidence = event
            .map(|e| {
                // Look for common verification keywords
                let payload = e.payload.to_lowercase();
                payload.contains("pass") || payload.contains("tests") || payload.contains("lint")
            })
            .unwrap_or(false);

        super::AssertionBuilder::new("build.done has verification evidence")
            .expected("Payload with test/lint/pass keywords")
            .actual(match event {
                Some(e) => format!("Payload: {}", truncate(&e.payload, 50)),
                None => "Event not found".to_string(),
            })
            .build()
            .with_passed(has_evidence)
    }
}

/// Extension trait for with_passed (duplicated here to avoid cross-module issues)
trait AssertionExt {
    fn with_passed(self, passed: bool) -> Self;
}

impl AssertionExt for crate::models::Assertion {
    fn with_passed(mut self, passed: bool) -> Self {
        self.passed = passed;
        self
    }
}

/// Truncates a string to the given length, adding "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::EventRecord;
    use std::env;
    use std::fs;

    fn test_workspace(test_name: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!(
            "ralph-e2e-events-{}-{}",
            test_name,
            std::process::id()
        ))
    }

    fn cleanup_workspace(path: &std::path::PathBuf) {
        if path.exists() {
            fs::remove_dir_all(path).ok();
        }
    }

    fn mock_execution_result() -> crate::executor::ExecutionResult {
        crate::executor::ExecutionResult {
            exit_code: Some(0),
            stdout: "<event topic=\"test.event\">Test payload data</event>\nLOOP_COMPLETE"
                .to_string(),
            stderr: String::new(),
            duration: Duration::from_secs(5),
            scratchpad: None,
            events: vec![EventRecord {
                topic: "test.event".to_string(),
                payload: "Test payload data".to_string(),
            }],
            iterations: 1,
            termination_reason: Some("LOOP_COMPLETE".to_string()),
            timed_out: false,
        }
    }

    fn mock_backpressure_result() -> crate::executor::ExecutionResult {
        crate::executor::ExecutionResult {
            exit_code: Some(0),
            stdout: "<event topic=\"build.done\">tests: pass, lint: pass</event>\nLOOP_COMPLETE"
                .to_string(),
            stderr: String::new(),
            duration: Duration::from_secs(8),
            scratchpad: None,
            events: vec![EventRecord {
                topic: "build.done".to_string(),
                payload: "tests: pass, lint: pass".to_string(),
            }],
            iterations: 1,
            termination_reason: Some("LOOP_COMPLETE".to_string()),
            timed_out: false,
        }
    }

    // ========== ClaudeEventsScenario Tests ==========

    #[test]
    fn test_events_scenario_new() {
        let scenario = ClaudeEventsScenario::new();
        assert_eq!(scenario.id(), "claude-events");
        assert_eq!(scenario.backend(), Backend::Claude);
        assert_eq!(scenario.tier(), "Tier 3: Events");
    }

    #[test]
    fn test_events_scenario_default() {
        let scenario = ClaudeEventsScenario::default();
        assert_eq!(scenario.id(), "claude-events");
    }

    #[test]
    fn test_events_scenario_description() {
        let scenario = ClaudeEventsScenario::new();
        assert!(scenario.description().contains("event"));
    }

    #[test]
    fn test_events_setup_creates_config() {
        let workspace = test_workspace("events-setup");
        fs::create_dir_all(&workspace).unwrap();

        let scenario = ClaudeEventsScenario::new();
        let config = scenario.setup(&workspace).unwrap();

        // Verify ralph.yml was created
        let config_path = workspace.join("ralph.yml");
        assert!(config_path.exists(), "ralph.yml should exist");

        // Verify content
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("backend: claude"));
        assert!(content.contains("max_iterations: 1"));

        // Verify .agent directory was created
        assert!(workspace.join(".agent").exists());

        // Verify config struct
        assert_eq!(config.max_iterations, 1);
        assert_eq!(config.timeout, Duration::from_secs(300));

        cleanup_workspace(&workspace);
    }

    #[test]
    fn test_events_event_has_payload_passed() {
        let scenario = ClaudeEventsScenario::new();
        let result = mock_execution_result();
        let assertion = scenario.event_has_payload(&result, "test.event", "Test payload");
        assert!(
            assertion.passed,
            "Should pass when payload contains expected substring"
        );
    }

    #[test]
    fn test_events_event_has_payload_failed_wrong_substring() {
        let scenario = ClaudeEventsScenario::new();
        let result = mock_execution_result();
        let assertion = scenario.event_has_payload(&result, "test.event", "nonexistent");
        assert!(
            !assertion.passed,
            "Should fail when payload doesn't contain substring"
        );
    }

    #[test]
    fn test_events_event_has_payload_failed_wrong_topic() {
        let scenario = ClaudeEventsScenario::new();
        let result = mock_execution_result();
        let assertion = scenario.event_has_payload(&result, "wrong.topic", "Test payload");
        assert!(!assertion.passed, "Should fail when event topic not found");
    }

    #[test]
    fn test_events_event_has_payload_failed_no_events() {
        let scenario = ClaudeEventsScenario::new();
        let mut result = mock_execution_result();
        result.events = vec![];
        let assertion = scenario.event_has_payload(&result, "test.event", "Test payload");
        assert!(!assertion.passed, "Should fail when no events present");
    }

    // ========== ClaudeBackpressureScenario Tests ==========

    #[test]
    fn test_backpressure_scenario_new() {
        let scenario = ClaudeBackpressureScenario::new();
        assert_eq!(scenario.id(), "claude-backpressure");
        assert_eq!(scenario.backend(), Backend::Claude);
        assert_eq!(scenario.tier(), "Tier 3: Events");
    }

    #[test]
    fn test_backpressure_scenario_default() {
        let scenario = ClaudeBackpressureScenario::default();
        assert_eq!(scenario.id(), "claude-backpressure");
    }

    #[test]
    fn test_backpressure_scenario_description() {
        let scenario = ClaudeBackpressureScenario::new();
        assert!(scenario.description().contains("backpressure"));
    }

    #[test]
    fn test_backpressure_setup_creates_config() {
        let workspace = test_workspace("backpressure-setup");
        fs::create_dir_all(&workspace).unwrap();

        let scenario = ClaudeBackpressureScenario::new();
        let config = scenario.setup(&workspace).unwrap();

        // Verify ralph.yml was created
        let config_path = workspace.join("ralph.yml");
        assert!(config_path.exists(), "ralph.yml should exist");

        // Verify content
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("backend: claude"));
        assert!(content.contains("max_iterations: 2"));

        // Verify config struct
        assert_eq!(config.max_iterations, 2);
        assert_eq!(config.timeout, Duration::from_secs(300));

        cleanup_workspace(&workspace);
    }

    #[test]
    fn test_backpressure_build_done_has_evidence_passed() {
        let scenario = ClaudeBackpressureScenario::new();
        let result = mock_backpressure_result();
        let assertion = scenario.build_done_has_evidence(&result);
        assert!(
            assertion.passed,
            "Should pass when build.done has verification evidence"
        );
    }

    #[test]
    fn test_backpressure_build_done_has_evidence_passed_tests_keyword() {
        let scenario = ClaudeBackpressureScenario::new();
        let mut result = mock_backpressure_result();
        result.events = vec![EventRecord {
            topic: "build.done".to_string(),
            payload: "all tests passed".to_string(),
        }];
        let assertion = scenario.build_done_has_evidence(&result);
        assert!(assertion.passed, "Should pass with 'tests' keyword");
    }

    #[test]
    fn test_backpressure_build_done_has_evidence_passed_lint_keyword() {
        let scenario = ClaudeBackpressureScenario::new();
        let mut result = mock_backpressure_result();
        result.events = vec![EventRecord {
            topic: "build.done".to_string(),
            payload: "lint clean".to_string(),
        }];
        let assertion = scenario.build_done_has_evidence(&result);
        assert!(assertion.passed, "Should pass with 'lint' keyword");
    }

    #[test]
    fn test_backpressure_build_done_has_evidence_failed_no_keywords() {
        let scenario = ClaudeBackpressureScenario::new();
        let mut result = mock_backpressure_result();
        result.events = vec![EventRecord {
            topic: "build.done".to_string(),
            payload: "completed successfully".to_string(),
        }];
        let assertion = scenario.build_done_has_evidence(&result);
        assert!(
            !assertion.passed,
            "Should fail without verification keywords"
        );
    }

    #[test]
    fn test_backpressure_build_done_has_evidence_failed_no_event() {
        let scenario = ClaudeBackpressureScenario::new();
        let mut result = mock_backpressure_result();
        result.events = vec![];
        let assertion = scenario.build_done_has_evidence(&result);
        assert!(
            !assertion.passed,
            "Should fail when build.done event missing"
        );
    }

    #[test]
    fn test_backpressure_build_done_has_evidence_failed_wrong_topic() {
        let scenario = ClaudeBackpressureScenario::new();
        let mut result = mock_backpressure_result();
        result.events = vec![EventRecord {
            topic: "other.event".to_string(),
            payload: "tests: pass".to_string(),
        }];
        let assertion = scenario.build_done_has_evidence(&result);
        assert!(
            !assertion.passed,
            "Should fail when build.done topic not found"
        );
    }

    // ========== Integration Tests (ignored by default) ==========

    #[tokio::test]
    #[ignore = "requires live backend"]
    async fn test_events_full_run() {
        let workspace = test_workspace("events-full");
        fs::create_dir_all(&workspace).unwrap();

        let scenario = ClaudeEventsScenario::new();
        let config = scenario.setup(&workspace).unwrap();

        let executor = RalphExecutor::new(workspace.clone());
        let result = scenario.run(&executor, &config).await;

        cleanup_workspace(&workspace);

        let result = result.expect("run should succeed");
        println!("Assertions:");
        for a in &result.assertions {
            println!(
                "  {} - {}: {} (expected: {})",
                if a.passed { "✅" } else { "❌" },
                a.name,
                a.actual,
                a.expected
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires live backend"]
    async fn test_backpressure_full_run() {
        let workspace = test_workspace("backpressure-full");
        fs::create_dir_all(&workspace).unwrap();

        let scenario = ClaudeBackpressureScenario::new();
        let config = scenario.setup(&workspace).unwrap();

        let executor = RalphExecutor::new(workspace.clone());
        let result = scenario.run(&executor, &config).await;

        cleanup_workspace(&workspace);

        let result = result.expect("run should succeed");
        println!("Assertions:");
        for a in &result.assertions {
            println!(
                "  {} - {}: {} (expected: {})",
                if a.passed { "✅" } else { "❌" },
                a.name,
                a.actual,
                a.expected
            );
        }
    }
}
