//! # ralph-adapters
//!
//! Agent adapters for the Ralph Orchestrator framework.
//!
//! This crate provides implementations for various AI agent backends:
//! - Claude (Anthropic)
//! - Gemini (Google)
//! - Codex (OpenAI)
//! - Amp
//! - Custom commands
//!
//! Each adapter implements the common CLI executor interface.

mod cli_backend;
mod cli_executor;

pub use cli_backend::{CliBackend, PromptMode};
pub use cli_executor::{CliExecutor, ExecutionResult};
