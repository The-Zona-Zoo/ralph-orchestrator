//! Event parsing from CLI output.
//!
//! Parses XML-style event tags from agent output:
//! ```text
//! <event topic="impl.done">payload</event>
//! <event topic="handoff" target="reviewer">payload</event>
//! ```

use ralph_proto::{Event, HatId};

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

    /// Checks if output contains the completion promise.
    pub fn contains_promise(output: &str, promise: &str) -> bool {
        output.contains(promise)
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
}
