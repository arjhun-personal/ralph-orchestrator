//! Event parsing from CLI output.
//!
//! Parses XML-style event tags from agent output:
//! ```text
//! <event topic="impl.done">payload</event>
//! <event topic="handoff" target="reviewer">payload</event>
//! ```

use ralph_proto::{Event, HatId};

/// Evidence of backpressure checks for build.done events.
#[derive(Debug, Clone, PartialEq)]
pub struct BackpressureEvidence {
    pub tests_passed: bool,
    pub lint_passed: bool,
    pub typecheck_passed: bool,
}

impl BackpressureEvidence {
    /// Returns true if all checks passed.
    pub fn all_passed(&self) -> bool {
        self.tests_passed && self.lint_passed && self.typecheck_passed
    }
}

/// Parser for extracting events from CLI output.
#[derive(Debug, Default)]
pub struct EventParser {
    /// The source hat ID to attach to parsed events.
    source: Option<HatId>,
}

impl EventParser {
    /// Creates a new event parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the source hat for parsed events.
    pub fn with_source(mut self, source: impl Into<HatId>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Parses events from CLI output text.
    ///
    /// Returns a list of parsed events.
    pub fn parse(&self, output: &str) -> Vec<Event> {
        let mut events = Vec::new();
        let mut remaining = output;

        while let Some(start_idx) = remaining.find("<event ") {
            let after_start = &remaining[start_idx..];

            // Find the end of the opening tag
            let Some(tag_end) = after_start.find('>') else {
                remaining = &remaining[start_idx + 7..];
                continue;
            };

            let opening_tag = &after_start[..tag_end + 1];

            // Parse attributes from opening tag
            let topic = Self::extract_attr(opening_tag, "topic");
            let target = Self::extract_attr(opening_tag, "target");

            let Some(topic) = topic else {
                remaining = &remaining[start_idx + tag_end + 1..];
                continue;
            };

            // Find the closing tag
            let content_start = &after_start[tag_end + 1..];
            let Some(close_idx) = content_start.find("</event>") else {
                remaining = &remaining[start_idx + tag_end + 1..];
                continue;
            };

            let payload = content_start[..close_idx].trim().to_string();

            let mut event = Event::new(topic, payload);

            if let Some(source) = &self.source {
                event = event.with_source(source.clone());
            }

            if let Some(target) = target {
                event = event.with_target(target);
            }

            events.push(event);

            // Move past this event
            let total_consumed = start_idx + tag_end + 1 + close_idx + 8; // 8 = "</event>".len()
            remaining = &remaining[total_consumed..];
        }

        events
    }

    /// Extracts an attribute value from an XML-like tag.
    fn extract_attr(tag: &str, attr: &str) -> Option<String> {
        let pattern = format!("{attr}=\"");
        let start = tag.find(&pattern)?;
        let value_start = start + pattern.len();
        let rest = &tag[value_start..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    }

    /// Parses backpressure evidence from build.done event payload.
    ///
    /// Expected format:
    /// ```text
    /// tests: pass
    /// lint: pass
    /// typecheck: pass
    /// ```
    pub fn parse_backpressure_evidence(payload: &str) -> Option<BackpressureEvidence> {
        let tests_passed = payload.contains("tests: pass");
        let lint_passed = payload.contains("lint: pass");
        let typecheck_passed = payload.contains("typecheck: pass");

        // Only return evidence if at least one check is mentioned
        if payload.contains("tests:") || payload.contains("lint:") || payload.contains("typecheck:") {
            Some(BackpressureEvidence {
                tests_passed,
                lint_passed,
                typecheck_passed,
            })
        } else {
            None
        }
    }

    /// Checks if output contains the completion promise.
    ///
    /// Per spec: The promise must appear in the agent's final output,
    /// not inside an `<event>` tag payload. This function:
    /// 1. Returns false if the promise appears inside ANY event tag
    ///    (prevents accidental completion when agents discuss the promise)
    /// 2. Otherwise, checks for the promise in the stripped output
    pub fn contains_promise(output: &str, promise: &str) -> bool {
        // Safety check: if promise appears inside any event tag, never complete
        if Self::promise_in_event_tags(output, promise) {
            return false;
        }
        let stripped = Self::strip_event_tags(output);
        stripped.contains(promise)
    }

    /// Checks if the promise appears inside any event tag payload.
    pub fn promise_in_event_tags(output: &str, promise: &str) -> bool {
        let mut remaining = output;

        while let Some(start_idx) = remaining.find("<event ") {
            let after_start = &remaining[start_idx..];

            // Find the end of the opening tag
            let Some(tag_end) = after_start.find('>') else {
                remaining = &remaining[start_idx + 7..];
                continue;
            };

            // Find the closing tag
            let content_start = &after_start[tag_end + 1..];
            let Some(close_idx) = content_start.find("</event>") else {
                remaining = &remaining[start_idx + tag_end + 1..];
                continue;
            };

            let payload = &content_start[..close_idx];
            if payload.contains(promise) {
                return true;
            }

            // Move past this event
            let total_consumed = start_idx + tag_end + 1 + close_idx + 8;
            remaining = &remaining[total_consumed..];
        }

        false
    }

    /// Strips all `<event ...>...</event>` blocks from output.
    ///
    /// Returns the output with event tags removed, leaving only
    /// the "final output" text that should be checked for promises.
    fn strip_event_tags(output: &str) -> String {
        let mut result = String::with_capacity(output.len());
        let mut remaining = output;

        while let Some(start_idx) = remaining.find("<event ") {
            // Add everything before this event tag
            result.push_str(&remaining[..start_idx]);

            let after_start = &remaining[start_idx..];

            // Find the closing tag
            if let Some(close_idx) = after_start.find("</event>") {
                // Skip past the entire event block
                remaining = &after_start[close_idx + 8..]; // 8 = "</event>".len()
            } else {
                // Malformed: no closing tag, keep the rest and stop
                result.push_str(after_start);
                remaining = "";
                break;
            }
        }

        // Add any remaining content after the last event
        result.push_str(remaining);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_event() {
        let output = r#"
Some preamble text.
<event topic="impl.done">
Implemented the authentication module.
</event>
Some trailing text.
"#;
        let parser = EventParser::new();
        let events = parser.parse(output);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].topic.as_str(), "impl.done");
        assert!(events[0].payload.contains("authentication module"));
    }

    #[test]
    fn test_parse_event_with_target() {
        let output = r#"<event topic="handoff" target="reviewer">Please review</event>"#;
        let parser = EventParser::new();
        let events = parser.parse(output);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target.as_ref().unwrap().as_str(), "reviewer");
    }

    #[test]
    fn test_parse_multiple_events() {
        let output = r#"
<event topic="impl.started">Starting work</event>
Working on implementation...
<event topic="impl.done">Finished</event>
"#;
        let parser = EventParser::new();
        let events = parser.parse(output);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].topic.as_str(), "impl.started");
        assert_eq!(events[1].topic.as_str(), "impl.done");
    }

    #[test]
    fn test_parse_with_source() {
        let output = r#"<event topic="impl.done">Done</event>"#;
        let parser = EventParser::new().with_source("implementer");
        let events = parser.parse(output);

        assert_eq!(events[0].source.as_ref().unwrap().as_str(), "implementer");
    }

    #[test]
    fn test_no_events() {
        let output = "Just regular output with no events.";
        let parser = EventParser::new();
        let events = parser.parse(output);

        assert!(events.is_empty());
    }

    #[test]
    fn test_contains_promise() {
        assert!(EventParser::contains_promise("LOOP_COMPLETE", "LOOP_COMPLETE"));
        assert!(EventParser::contains_promise("prefix LOOP_COMPLETE suffix", "LOOP_COMPLETE"));
        assert!(!EventParser::contains_promise("No promise here", "LOOP_COMPLETE"));
    }

    #[test]
    fn test_contains_promise_ignores_event_payloads() {
        // Promise inside event payload should NOT be detected
        let output = r#"<event topic="build.task">Fix LOOP_COMPLETE detection</event>"#;
        assert!(!EventParser::contains_promise(output, "LOOP_COMPLETE"));

        // Promise inside event with acceptance criteria mentioning LOOP_COMPLETE
        let output = r#"<event topic="build.task">
## Task: Fix completion promise detection
- Given LOOP_COMPLETE appears inside an event tag
- Then it should be ignored
</event>"#;
        assert!(!EventParser::contains_promise(output, "LOOP_COMPLETE"));
    }

    #[test]
    fn test_contains_promise_detects_outside_events() {
        // Promise outside event tags should be detected
        let output = r#"<event topic="build.done">Task complete</event>
All done! LOOP_COMPLETE"#;
        assert!(EventParser::contains_promise(output, "LOOP_COMPLETE"));

        // Promise before event tags
        let output = r#"LOOP_COMPLETE
<event topic="summary">Final summary</event>"#;
        assert!(EventParser::contains_promise(output, "LOOP_COMPLETE"));
    }

    #[test]
    fn test_contains_promise_mixed_content() {
        // Promise only in event payload, not in surrounding text
        let output = r#"Working on task...
<event topic="build.task">Fix LOOP_COMPLETE bug</event>
Still working..."#;
        assert!(!EventParser::contains_promise(output, "LOOP_COMPLETE"));

        // Promise in both event and surrounding text - should NOT complete
        // because promise appears inside an event tag (safety mechanism)
        let output = r#"All tasks done. LOOP_COMPLETE
<event topic="summary">Completed LOOP_COMPLETE task</event>"#;
        assert!(!EventParser::contains_promise(output, "LOOP_COMPLETE"));
    }

    #[test]
    fn test_promise_in_event_tags() {
        // Promise inside event payload
        let output = r#"<event topic="build.task">Fix LOOP_COMPLETE bug</event>"#;
        assert!(EventParser::promise_in_event_tags(output, "LOOP_COMPLETE"));

        // Promise not in any event
        let output = r#"<event topic="build.done">Task complete</event>"#;
        assert!(!EventParser::promise_in_event_tags(output, "LOOP_COMPLETE"));

        // No events at all
        let output = "Just regular text with LOOP_COMPLETE";
        assert!(!EventParser::promise_in_event_tags(output, "LOOP_COMPLETE"));

        // Multiple events, promise in second
        let output = r#"<event topic="a">first</event>
<event topic="b">contains LOOP_COMPLETE</event>"#;
        assert!(EventParser::promise_in_event_tags(output, "LOOP_COMPLETE"));
    }

    #[test]
    fn test_strip_event_tags() {
        // Single event
        let output = r#"before <event topic="test">payload</event> after"#;
        let stripped = EventParser::strip_event_tags(output);
        assert_eq!(stripped, "before  after");
        assert!(!stripped.contains("payload"));

        // Multiple events
        let output = r#"start <event topic="a">one</event> middle <event topic="b">two</event> end"#;
        let stripped = EventParser::strip_event_tags(output);
        assert_eq!(stripped, "start  middle  end");

        // No events
        let output = "just plain text";
        let stripped = EventParser::strip_event_tags(output);
        assert_eq!(stripped, "just plain text");
    }

    #[test]
    fn test_parse_backpressure_evidence_all_pass() {
        let payload = "tests: pass\nlint: pass\ntypecheck: pass";
        let evidence = EventParser::parse_backpressure_evidence(payload).unwrap();
        assert!(evidence.tests_passed);
        assert!(evidence.lint_passed);
        assert!(evidence.typecheck_passed);
        assert!(evidence.all_passed());
    }

    #[test]
    fn test_parse_backpressure_evidence_some_fail() {
        let payload = "tests: pass\nlint: fail\ntypecheck: pass";
        let evidence = EventParser::parse_backpressure_evidence(payload).unwrap();
        assert!(evidence.tests_passed);
        assert!(!evidence.lint_passed);
        assert!(evidence.typecheck_passed);
        assert!(!evidence.all_passed());
    }

    #[test]
    fn test_parse_backpressure_evidence_missing() {
        let payload = "Task completed successfully";
        let evidence = EventParser::parse_backpressure_evidence(payload);
        assert!(evidence.is_none());
    }

    #[test]
    fn test_parse_backpressure_evidence_partial() {
        let payload = "tests: pass\nSome other text";
        let evidence = EventParser::parse_backpressure_evidence(payload).unwrap();
        assert!(evidence.tests_passed);
        assert!(!evidence.lint_passed);
        assert!(!evidence.typecheck_passed);
        assert!(!evidence.all_passed());
    }
}
