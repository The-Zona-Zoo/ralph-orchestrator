//! Configuration types for the Ralph Orchestrator.

use ralph_proto::Topic;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Top-level configuration for Ralph Orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphConfig {
    /// Execution mode: "single" or "multi".
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Event loop configuration.
    #[serde(default)]
    pub event_loop: EventLoopConfig,

    /// CLI backend configuration.
    #[serde(default)]
    pub cli: CliConfig,

    /// Hat definitions for multi-hat mode.
    #[serde(default)]
    pub hats: HashMap<String, HatConfig>,
}

fn default_mode() -> String {
    "single".to_string()
}

impl Default for RalphConfig {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            event_loop: EventLoopConfig::default(),
            cli: CliConfig::default(),
            hats: HashMap::new(),
        }
    }
}

impl RalphConfig {
    /// Loads configuration from a YAML file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Returns true if this is single-hat mode.
    pub fn is_single_mode(&self) -> bool {
        self.mode == "single"
    }
}

/// Event loop configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLoopConfig {
    /// Path to the prompt file.
    #[serde(default = "default_prompt_file")]
    pub prompt_file: String,

    /// String that signals loop completion.
    #[serde(default = "default_completion_promise")]
    pub completion_promise: String,

    /// Maximum number of iterations before timeout.
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,

    /// Maximum runtime in seconds.
    #[serde(default = "default_max_runtime")]
    pub max_runtime_seconds: u64,

    /// Maximum cost in USD before stopping.
    pub max_cost_usd: Option<f64>,

    /// Stop after this many consecutive failures.
    #[serde(default = "default_max_failures")]
    pub max_consecutive_failures: u32,

    /// Create checkpoint commit every N iterations.
    #[serde(default = "default_checkpoint_interval")]
    pub checkpoint_interval: u32,

    /// Starting hat for multi-hat mode.
    pub starting_hat: Option<String>,
}

fn default_prompt_file() -> String {
    "PROMPT.md".to_string()
}

fn default_completion_promise() -> String {
    "LOOP_COMPLETE".to_string()
}

fn default_max_iterations() -> u32 {
    100
}

fn default_max_runtime() -> u64 {
    14400 // 4 hours
}

fn default_max_failures() -> u32 {
    5
}

fn default_checkpoint_interval() -> u32 {
    5
}

impl Default for EventLoopConfig {
    fn default() -> Self {
        Self {
            prompt_file: default_prompt_file(),
            completion_promise: default_completion_promise(),
            max_iterations: default_max_iterations(),
            max_runtime_seconds: default_max_runtime(),
            max_cost_usd: None,
            max_consecutive_failures: default_max_failures(),
            checkpoint_interval: default_checkpoint_interval(),
            starting_hat: None,
        }
    }
}

/// CLI backend configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Backend to use: "claude", "gemini", "codex", "amp", or "custom".
    #[serde(default = "default_backend")]
    pub backend: String,

    /// Custom command (for backend: "custom").
    pub command: Option<String>,

    /// How to pass prompts: "arg" or "stdin".
    #[serde(default = "default_prompt_mode")]
    pub prompt_mode: String,
}

fn default_backend() -> String {
    "claude".to_string()
}

fn default_prompt_mode() -> String {
    "arg".to_string()
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            command: None,
            prompt_mode: default_prompt_mode(),
        }
    }
}

/// Configuration for a single hat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HatConfig {
    /// Human-readable name for the hat.
    pub name: String,

    /// Topic patterns this hat subscribes to.
    #[serde(default)]
    pub subscriptions: Vec<String>,

    /// Topics this hat publishes.
    #[serde(default)]
    pub publishes: Vec<String>,

    /// Instructions prepended to prompts.
    #[serde(default)]
    pub instructions: String,
}

impl HatConfig {
    /// Converts subscription strings to Topic objects.
    pub fn subscription_topics(&self) -> Vec<Topic> {
        self.subscriptions.iter().map(|s| Topic::new(s)).collect()
    }

    /// Converts publish strings to Topic objects.
    pub fn publish_topics(&self) -> Vec<Topic> {
        self.publishes.iter().map(|s| Topic::new(s)).collect()
    }
}

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RalphConfig::default();
        assert_eq!(config.mode, "single");
        assert!(config.is_single_mode());
        assert_eq!(config.event_loop.max_iterations, 100);
    }

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
mode: "multi"
event_loop:
  prompt_file: "TASK.md"
  completion_promise: "DONE"
  max_iterations: 50
cli:
  backend: "claude"
hats:
  implementer:
    name: "Implementer"
    subscriptions: ["task.*", "review.done"]
    publishes: ["impl.done"]
    instructions: "You are the implementation agent."
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.mode, "multi");
        assert!(!config.is_single_mode());
        assert_eq!(config.event_loop.prompt_file, "TASK.md");
        assert_eq!(config.hats.len(), 1);

        let hat = config.hats.get("implementer").unwrap();
        assert_eq!(hat.subscriptions.len(), 2);
    }
}
