//! # tripartite-rs
//!
//! A generic multi-agent consensus system for Rust.
//!
//! ## Overview
//!
//! This crate provides a framework for coordinating multiple AI agents to reach
//! consensus before responding. It implements a weighted voting mechanism with
//! multi-round coordination.
//!
//! ## Core Concepts
//!
//! - **Agent**: A specialized AI agent that processes inputs and produces outputs
//! - **Consensus Engine**: Coordinates multiple agents to reach agreement
//! - **Voting**: Weighted confidence-based voting mechanism
//! - **Multi-Round**: Automatic revision when consensus isn't reached
//!
//! ## Example
//!
//! ```rust,no_run
//! use tripartite::{ConsensusEngine, ConsensusConfig, Agent, AgentInput, AgentOutput};
//! use async_trait::async_trait;
//!
//! struct MyAgent;
//!
//! #[async_trait]
//! impl Agent for MyAgent {
//!     fn name(&self) -> &str { "MyAgent" }
//!     fn role(&self) -> &str { "Example agent" }
//!
//!     async fn process(&self, input: AgentInput) -> tripartite::Result<AgentOutput> {
//!         Ok(AgentOutput::new("MyAgent", "Response".to_string(), 0.9))
//!     }
//!
//!     fn is_ready(&self) -> bool { true }
//!     fn model(&self) -> &str { "my-model" }
//! }
//! ```

pub mod agent;
pub mod consensus;
pub mod error;
pub mod manifest;

// Re-exports
pub use agent::{Agent, AgentConfig, AgentInput, AgentOutput, ConsensusVote};
pub use consensus::{
    AgentWeights, ConsensusConfig, ConsensusEngine, ConsensusOutcome, ConsensusResult, Verdict,
    Votes,
};
pub use error::{Error, Result};
pub use manifest::{A2AManifest, ConversationTurn, ManifestFlags};
