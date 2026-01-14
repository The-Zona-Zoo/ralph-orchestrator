//! Event reader for consuming events from `.agent/events.jsonl`.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use tracing::warn;

/// A simplified event for reading from JSONL.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    pub ts: String,
}

/// Reads new events from `.agent/events.jsonl` since last read.
pub struct EventReader {
    path: PathBuf,
    position: u64,
}

impl EventReader {
    /// Creates a new event reader for the given path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            position: 0,
        }
    }

    /// Reads new events since the last read.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read.
    pub fn read_new_events(&mut self) -> std::io::Result<Vec<Event>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(self.position))?;

        let reader = BufReader::new(file);
        let mut events = Vec::new();
        let mut current_pos = self.position;

        for line in reader.lines() {
            let line = line?;
            let line_bytes = line.len() as u64 + 1; // +1 for newline

            if line.trim().is_empty() {
                current_pos += line_bytes;
                continue;
            }

            match serde_json::from_str::<Event>(&line) {
                Ok(event) => events.push(event),
                Err(e) => {
                    warn!(error = %e, line = %line, "Skipping corrupt JSON line");
                }
            }

            current_pos += line_bytes;
        }

        self.position = current_pos;
        Ok(events)
    }

    /// Returns the current file position.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Resets the position to the start of the file.
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_new_events() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"topic":"test","payload":"hello","ts":"2024-01-01T00:00:00Z"}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"topic":"test2","ts":"2024-01-01T00:00:01Z"}}"#
        )
        .unwrap();
        file.flush().unwrap();

        let mut reader = EventReader::new(file.path());
        let events = reader.read_new_events().unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].topic, "test");
        assert_eq!(events[0].payload, Some("hello".to_string()));
        assert_eq!(events[1].topic, "test2");
        assert_eq!(events[1].payload, None);
    }

    #[test]
    fn test_tracks_position() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"topic":"first","ts":"2024-01-01T00:00:00Z"}}"#
        )
        .unwrap();
        file.flush().unwrap();

        let mut reader = EventReader::new(file.path());
        let events = reader.read_new_events().unwrap();
        assert_eq!(events.len(), 1);

        // Add more events
        writeln!(
            file,
            r#"{{"topic":"second","ts":"2024-01-01T00:00:01Z"}}"#
        )
        .unwrap();
        file.flush().unwrap();

        // Should only read new events
        let events = reader.read_new_events().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].topic, "second");
    }

    #[test]
    fn test_missing_file() {
        let mut reader = EventReader::new("/nonexistent/path.jsonl");
        let events = reader.read_new_events().unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_skips_corrupt_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"topic":"good","ts":"2024-01-01T00:00:00Z"}}"#
        )
        .unwrap();
        writeln!(file, r#"{{corrupt json}}"#).unwrap();
        writeln!(
            file,
            r#"{{"topic":"also_good","ts":"2024-01-01T00:00:01Z"}}"#
        )
        .unwrap();
        file.flush().unwrap();

        let mut reader = EventReader::new(file.path());
        let events = reader.read_new_events().unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].topic, "good");
        assert_eq!(events[1].topic, "also_good");
    }

    #[test]
    fn test_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let mut reader = EventReader::new(file.path());
        let events = reader.read_new_events().unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_reset_position() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"topic":"test","ts":"2024-01-01T00:00:00Z"}}"#
        )
        .unwrap();
        file.flush().unwrap();

        let mut reader = EventReader::new(file.path());
        reader.read_new_events().unwrap();
        assert!(reader.position() > 0);

        reader.reset();
        assert_eq!(reader.position(), 0);

        let events = reader.read_new_events().unwrap();
        assert_eq!(events.len(), 1);
    }
}
