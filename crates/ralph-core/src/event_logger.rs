//! Event logging for debugging and post-mortem analysis.
//!
//! Logs all events to `.agent/events.jsonl` as specified in the event-loop spec.
//! The observer pattern allows hooking into the event bus without modifying routing.

use ralph_proto::{Event, HatId};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// A logged event record for debugging.
///
/// Matches the spec format:
/// ```jsonl
/// {"ts":"2024-01-15T10:23:45Z","iteration":1,"hat":"loop","topic":"task.start","triggered":"planner","payload":"..."}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    /// ISO 8601 timestamp.
    pub ts: String,

    /// Loop iteration number.
    pub iteration: u32,

    /// Hat that was active when event was published.
    pub hat: String,

    /// Event topic.
    pub topic: String,

    /// Hat that will be triggered by this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggered: Option<String>,

    /// Event content (truncated if large).
    pub payload: String,

    /// How many times this task has blocked (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_count: Option<u32>,
}

impl EventRecord {
    /// Maximum payload length before truncation.
    const MAX_PAYLOAD_LEN: usize = 500;

    /// Creates a new event record.
    pub fn new(
        iteration: u32,
        hat: impl Into<String>,
        event: &Event,
        triggered: Option<&HatId>,
    ) -> Self {
        let payload = if event.payload.len() > Self::MAX_PAYLOAD_LEN {
            format!(
                "{}... [truncated, {} chars total]",
                &event.payload[..Self::MAX_PAYLOAD_LEN],
                event.payload.len()
            )
        } else {
            event.payload.clone()
        };

        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            iteration,
            hat: hat.into(),
            topic: event.topic.to_string(),
            triggered: triggered.map(|h| h.to_string()),
            payload,
            blocked_count: None,
        }
    }

    /// Sets the blocked count for this record.
    pub fn with_blocked_count(mut self, count: u32) -> Self {
        self.blocked_count = Some(count);
        self
    }
}

/// Logger that writes events to a JSONL file.
pub struct EventLogger {
    /// Path to the events file.
    path: PathBuf,

    /// File handle for appending.
    file: Option<File>,
}

impl EventLogger {
    /// Default path for the events file.
    pub const DEFAULT_PATH: &'static str = ".agent/events.jsonl";

    /// Creates a new event logger.
    ///
    /// The `.agent/` directory is created if it doesn't exist.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            file: None,
        }
    }

    /// Creates a logger with the default path.
    pub fn default_path() -> Self {
        Self::new(Self::DEFAULT_PATH)
    }

    /// Ensures the parent directory exists and opens the file.
    fn ensure_open(&mut self) -> std::io::Result<&mut File> {
        if self.file.is_none() {
            if let Some(parent) = self.path.parent() {
                fs::create_dir_all(parent)?;
            }
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;
            self.file = Some(file);
        }
        Ok(self.file.as_mut().unwrap())
    }

    /// Logs an event record.
    pub fn log(&mut self, record: &EventRecord) -> std::io::Result<()> {
        let file = self.ensure_open()?;
        let json = serde_json::to_string(record)?;
        writeln!(file, "{}", json)?;
        file.flush()?;
        debug!(topic = %record.topic, iteration = record.iteration, "Event logged");
        Ok(())
    }

    /// Convenience method to log an event directly.
    pub fn log_event(
        &mut self,
        iteration: u32,
        hat: &str,
        event: &Event,
        triggered: Option<&HatId>,
    ) -> std::io::Result<()> {
        let record = EventRecord::new(iteration, hat, event, triggered);
        self.log(&record)
    }

    /// Returns the path to the log file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Reader for event history files.
pub struct EventHistory {
    path: PathBuf,
}

impl EventHistory {
    /// Creates a new history reader.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Creates a reader for the default path.
    pub fn default_path() -> Self {
        Self::new(EventLogger::DEFAULT_PATH)
    }

    /// Returns true if the history file exists.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Reads all event records from the file.
    pub fn read_all(&self) -> std::io::Result<Vec<EventRecord>> {
        if !self.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str(&line) {
                Ok(record) => records.push(record),
                Err(e) => {
                    warn!(line = line_num + 1, error = %e, "Failed to parse event record");
                }
            }
        }

        Ok(records)
    }

    /// Reads the last N event records.
    pub fn read_last(&self, n: usize) -> std::io::Result<Vec<EventRecord>> {
        let all = self.read_all()?;
        let start = all.len().saturating_sub(n);
        Ok(all[start..].to_vec())
    }

    /// Reads events filtered by topic.
    pub fn filter_by_topic(&self, topic: &str) -> std::io::Result<Vec<EventRecord>> {
        let all = self.read_all()?;
        Ok(all.into_iter().filter(|r| r.topic == topic).collect())
    }

    /// Reads events filtered by iteration.
    pub fn filter_by_iteration(&self, iteration: u32) -> std::io::Result<Vec<EventRecord>> {
        let all = self.read_all()?;
        Ok(all
            .into_iter()
            .filter(|r| r.iteration == iteration)
            .collect())
    }

    /// Clears the event history file.
    pub fn clear(&self) -> std::io::Result<()> {
        if self.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_event(topic: &str, payload: &str) -> Event {
        Event::new(topic, payload)
    }

    #[test]
    fn test_log_and_read() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("events.jsonl");

        let mut logger = EventLogger::new(&path);

        // Log some events
        let event1 = make_event("task.start", "Starting task");
        let event2 = make_event("build.done", "Build complete");

        logger
            .log_event(1, "loop", &event1, Some(&HatId::new("planner")))
            .unwrap();
        logger
            .log_event(2, "builder", &event2, Some(&HatId::new("planner")))
            .unwrap();

        // Read them back
        let history = EventHistory::new(&path);
        let records = history.read_all().unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].topic, "task.start");
        assert_eq!(records[0].iteration, 1);
        assert_eq!(records[0].hat, "loop");
        assert_eq!(records[0].triggered, Some("planner".to_string()));
        assert_eq!(records[1].topic, "build.done");
    }

    #[test]
    fn test_read_last() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("events.jsonl");

        let mut logger = EventLogger::new(&path);

        for i in 1..=10 {
            let event = make_event("test", &format!("Event {}", i));
            logger.log_event(i, "hat", &event, None).unwrap();
        }

        let history = EventHistory::new(&path);
        let last_3 = history.read_last(3).unwrap();

        assert_eq!(last_3.len(), 3);
        assert_eq!(last_3[0].iteration, 8);
        assert_eq!(last_3[2].iteration, 10);
    }

    #[test]
    fn test_filter_by_topic() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("events.jsonl");

        let mut logger = EventLogger::new(&path);

        logger
            .log_event(1, "hat", &make_event("build.done", "a"), None)
            .unwrap();
        logger
            .log_event(2, "hat", &make_event("build.blocked", "b"), None)
            .unwrap();
        logger
            .log_event(3, "hat", &make_event("build.done", "c"), None)
            .unwrap();

        let history = EventHistory::new(&path);
        let blocked = history.filter_by_topic("build.blocked").unwrap();

        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].iteration, 2);
    }

    #[test]
    fn test_payload_truncation() {
        let long_payload = "x".repeat(1000);
        let event = make_event("test", &long_payload);
        let record = EventRecord::new(1, "hat", &event, None);

        assert!(record.payload.len() < 1000);
        assert!(record.payload.contains("[truncated"));
    }

    #[test]
    fn test_creates_parent_directory() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nested/dir/events.jsonl");

        let mut logger = EventLogger::new(&path);
        let event = make_event("test", "payload");
        logger.log_event(1, "hat", &event, None).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn test_empty_history() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.jsonl");

        let history = EventHistory::new(&path);
        assert!(!history.exists());

        let records = history.read_all().unwrap();
        assert!(records.is_empty());
    }
}
