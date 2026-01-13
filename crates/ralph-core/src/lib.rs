//! # ralph-core
//!
//! Core orchestration functionality for the Ralph Orchestrator framework.
//!
//! This crate provides:
//! - The main orchestration loop for coordinating multiple agents
//! - Configuration loading and management
//! - State management for agent sessions
//! - Message routing between agents

mod config;
mod event_loop;
mod event_parser;
mod hat_registry;
mod instructions;

pub use config::{CliConfig, EventLoopConfig, HatConfig, RalphConfig};
pub use event_loop::{EventLoop, LoopState, TerminationReason};
pub use event_parser::EventParser;
pub use hat_registry::HatRegistry;
pub use instructions::InstructionBuilder;
