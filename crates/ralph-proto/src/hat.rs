//! Hat types for agent personas.
//!
//! A hat defines how the CLI agent should behave for a given iteration.

use crate::Topic;
use serde::{Deserialize, Serialize};

/// Unique identifier for a hat.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HatId(String);

impl HatId {
    /// Creates a new hat ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for HatId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for HatId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Display for HatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A hat (persona) that defines agent behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hat {
    /// Unique identifier for this hat.
    pub id: HatId,

    /// Human-readable name for the hat.
    pub name: String,

    /// Topic patterns this hat subscribes to.
    pub subscriptions: Vec<Topic>,

    /// Topics this hat is expected to publish.
    pub publishes: Vec<Topic>,

    /// Instructions prepended to prompts for this hat.
    pub instructions: String,
}

impl Hat {
    /// Creates a new hat with the given ID and name.
    pub fn new(id: impl Into<HatId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            subscriptions: Vec::new(),
            publishes: Vec::new(),
            instructions: String::new(),
        }
    }

    /// Creates the default hat for single-hat mode.
    #[deprecated(note = "Use default_planner() and default_builder() instead")]
    pub fn default_single() -> Self {
        Self {
            id: HatId::new("default"),
            name: "Default".to_string(),
            subscriptions: vec![Topic::new("*")],
            publishes: vec![Topic::new("task.done")],
            instructions: String::new(),
        }
    }

    /// Creates the default planner hat.
    ///
    /// Per spec: Planner triggers on `task.start`, `build.done`, `build.blocked`
    /// and publishes `build.task`.
    pub fn default_planner() -> Self {
        Self {
            id: HatId::new("planner"),
            name: "Planner".to_string(),
            subscriptions: vec![
                Topic::new("task.start"),
                Topic::new("build.done"),
                Topic::new("build.blocked"),
            ],
            publishes: vec![Topic::new("build.task")],
            instructions: String::new(),
        }
    }

    /// Creates the default builder hat.
    ///
    /// Per spec: Builder triggers on `build.task` and publishes
    /// `build.done` or `build.blocked`.
    pub fn default_builder() -> Self {
        Self {
            id: HatId::new("builder"),
            name: "Builder".to_string(),
            subscriptions: vec![Topic::new("build.task")],
            publishes: vec![
                Topic::new("build.done"),
                Topic::new("build.blocked"),
            ],
            instructions: String::new(),
        }
    }

    /// Adds a subscription to this hat.
    #[must_use]
    pub fn subscribe(mut self, topic: impl Into<Topic>) -> Self {
        self.subscriptions.push(topic.into());
        self
    }

    /// Sets the instructions for this hat.
    #[must_use]
    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = instructions.into();
        self
    }

    /// Checks if this hat is subscribed to the given topic.
    pub fn is_subscribed(&self, topic: &Topic) -> bool {
        self.subscriptions.iter().any(|sub| sub.matches(topic))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_matching() {
        let hat = Hat::new("impl", "Implementer").subscribe("impl.*").subscribe("task.start");

        assert!(hat.is_subscribed(&Topic::new("impl.done")));
        assert!(hat.is_subscribed(&Topic::new("task.start")));
        assert!(!hat.is_subscribed(&Topic::new("review.done")));
    }

    #[test]
    #[allow(deprecated)]
    fn test_default_single_hat() {
        let hat = Hat::default_single();
        assert!(hat.is_subscribed(&Topic::new("anything")));
        assert!(hat.is_subscribed(&Topic::new("impl.done")));
    }

    #[test]
    fn test_default_planner_hat() {
        let hat = Hat::default_planner();
        assert_eq!(hat.id.as_str(), "planner");
        assert!(hat.is_subscribed(&Topic::new("task.start")));
        assert!(hat.is_subscribed(&Topic::new("build.done")));
        assert!(hat.is_subscribed(&Topic::new("build.blocked")));
        assert!(!hat.is_subscribed(&Topic::new("build.task")));
    }

    #[test]
    fn test_default_builder_hat() {
        let hat = Hat::default_builder();
        assert_eq!(hat.id.as_str(), "builder");
        assert!(hat.is_subscribed(&Topic::new("build.task")));
        assert!(!hat.is_subscribed(&Topic::new("task.start")));
        assert!(!hat.is_subscribed(&Topic::new("build.done")));
    }
}
