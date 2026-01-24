//! Task provider resolution for native vs local task tracking.
//!
//! This module determines which task provider to use based on configuration
//! and the backend being used. When running with Claude Code, the native
//! task tools (TaskCreate, TaskUpdate, etc.) can be used instead of the
//! custom `ralph tools task` CLI commands.

use crate::config::TasksConfig;
use tracing::{debug, warn};

/// The resolved task provider for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskProvider {
    /// Use Claude Code's native task tools (TaskCreate, TaskUpdate, etc.)
    Native,
    /// Use `ralph tools task` commands and `.agent/tasks.jsonl`
    Local,
    /// Tasks are disabled
    Disabled,
}

impl TaskProvider {
    /// Returns true if this provider trusts the agent for completion verification.
    ///
    /// Native mode trusts the agent to verify all tasks are complete before
    /// signaling LOOP_COMPLETE. Local mode checks `.agent/tasks.jsonl`.
    pub fn trusts_agent(&self) -> bool {
        matches!(self, TaskProvider::Native)
    }
}

/// Resolves which task provider to use based on config and backend.
///
/// # Arguments
/// * `tasks_config` - The tasks configuration from ralph config
/// * `backend_name` - The name of the backend being used (e.g., "claude", "kiro")
///
/// # Returns
/// The resolved `TaskProvider` to use for this session.
///
/// # Examples
/// ```
/// use ralph_core::{TasksConfig, resolve_task_provider, TaskProvider};
///
/// // Auto mode with Claude backend → Native
/// let config = TasksConfig { enabled: true, provider: "auto".to_string() };
/// assert_eq!(resolve_task_provider(&config, "claude"), TaskProvider::Native);
///
/// // Auto mode with Kiro backend → Local
/// let config = TasksConfig { enabled: true, provider: "auto".to_string() };
/// assert_eq!(resolve_task_provider(&config, "kiro"), TaskProvider::Local);
/// ```
pub fn resolve_task_provider(tasks_config: &TasksConfig, backend_name: &str) -> TaskProvider {
    if !tasks_config.enabled {
        debug!("Tasks disabled in config");
        return TaskProvider::Disabled;
    }

    match tasks_config.provider.as_str() {
        "native" => {
            if is_claude_backend(backend_name) {
                debug!(provider = "native", "Using Claude Code native task tools");
                TaskProvider::Native
            } else {
                warn!(
                    provider = "native",
                    backend = backend_name,
                    "Native task tools not available for backend '{}'. Using local task tracking.",
                    backend_name
                );
                TaskProvider::Local
            }
        }
        "local" => {
            debug!(
                provider = "local",
                "Using local task tracking (.agent/tasks.jsonl)"
            );
            TaskProvider::Local
        }
        _ => {
            // Default to auto-detection behavior
            if is_claude_backend(backend_name) {
                debug!(
                    provider = "auto",
                    backend = backend_name,
                    "Auto-detected Claude backend, using native task tools"
                );
                TaskProvider::Native
            } else {
                debug!(
                    provider = "auto",
                    backend = backend_name,
                    "Auto-detected non-Claude backend, using local task tracking"
                );
                TaskProvider::Local
            }
        }
    }
}

/// Checks if the backend supports Claude Code's native task tools.
fn is_claude_backend(backend_name: &str) -> bool {
    backend_name == "claude"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(enabled: bool, provider: &str) -> TasksConfig {
        TasksConfig {
            enabled,
            provider: provider.to_string(),
        }
    }

    #[test]
    fn test_disabled_tasks() {
        assert_eq!(
            resolve_task_provider(&config(false, "auto"), "claude"),
            TaskProvider::Disabled
        );
        assert_eq!(
            resolve_task_provider(&config(false, "native"), "claude"),
            TaskProvider::Disabled
        );
    }

    #[test]
    fn test_native_with_claude() {
        assert_eq!(
            resolve_task_provider(&config(true, "native"), "claude"),
            TaskProvider::Native
        );
    }

    #[test]
    fn test_native_with_kiro_falls_back() {
        // Native requested but Kiro doesn't support it → falls back to Local
        assert_eq!(
            resolve_task_provider(&config(true, "native"), "kiro"),
            TaskProvider::Local
        );
    }

    #[test]
    fn test_native_with_gemini_falls_back() {
        assert_eq!(
            resolve_task_provider(&config(true, "native"), "gemini"),
            TaskProvider::Local
        );
    }

    #[test]
    fn test_local_always_local() {
        // Local mode always uses local, regardless of backend
        assert_eq!(
            resolve_task_provider(&config(true, "local"), "claude"),
            TaskProvider::Local
        );
        assert_eq!(
            resolve_task_provider(&config(true, "local"), "kiro"),
            TaskProvider::Local
        );
    }

    #[test]
    fn test_auto_with_claude() {
        assert_eq!(
            resolve_task_provider(&config(true, "auto"), "claude"),
            TaskProvider::Native
        );
    }

    #[test]
    fn test_auto_with_kiro() {
        assert_eq!(
            resolve_task_provider(&config(true, "auto"), "kiro"),
            TaskProvider::Local
        );
    }

    #[test]
    fn test_auto_with_gemini() {
        assert_eq!(
            resolve_task_provider(&config(true, "auto"), "gemini"),
            TaskProvider::Local
        );
    }

    #[test]
    fn test_unknown_provider_defaults_to_auto() {
        // Unknown provider value falls through to auto behavior
        assert_eq!(
            resolve_task_provider(&config(true, "unknown"), "claude"),
            TaskProvider::Native
        );
        assert_eq!(
            resolve_task_provider(&config(true, "unknown"), "kiro"),
            TaskProvider::Local
        );
    }

    #[test]
    fn test_trusts_agent() {
        assert!(TaskProvider::Native.trusts_agent());
        assert!(!TaskProvider::Local.trusts_agent());
        assert!(!TaskProvider::Disabled.trusts_agent());
    }
}
