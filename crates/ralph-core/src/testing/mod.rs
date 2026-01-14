//! Testing utilities for deterministic E2E tests.

pub mod mock_backend;
pub mod scenario;

pub use mock_backend::{MockBackend, ExecutionRecord};
pub use scenario::{Scenario, ScenarioRunner, ExecutionTrace};
