//! Agent trait and core types for the tripartite consensus system

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::manifest::A2AManifest;
use crate::Result;

// ============================================================================
// Core Agent Trait
// ============================================================================

/// Core agent trait that all consensus participants must implement
///
/// # Example
///
/// ```rust,no_run
/// use tripartite::{Agent, AgentInput, AgentOutput, Result};
/// use async_trait::async_trait;
///
/// struct MyAgent;
///
/// #[async_trait]
/// impl Agent for MyAgent {
///     fn name(&self) -> &str {
///         "MyAgent"
///     }
///
///     fn role(&self) -> &str {
///         "Custom agent implementation"
///     }
///
///     async fn process(&self, input: AgentInput) -> Result<AgentOutput> {
///         Ok(AgentOutput::new("MyAgent", "Response".to_string(), 0.9))
///     }
///
///     fn is_ready(&self) -> bool {
///         true
///     }
///
///     fn model(&self) -> &str {
///         "my-model"
///     }
/// }
/// ```
#[async_trait]
pub trait Agent: Send + Sync {
    /// Returns the agent's name for logging and display purposes
    fn name(&self) -> &str;

    /// Returns a brief description of the agent's role
    fn role(&self) -> &str;

    /// Process an input manifest and generate a response
    async fn process(&self, input: AgentInput) -> Result<AgentOutput>;

    /// Check if the agent is ready to process queries
    fn is_ready(&self) -> bool;

    /// Returns the identifier of the model this agent uses
    fn model(&self) -> &str;
}

// ============================================================================
// Agent Input/Output Types
// ============================================================================

/// Input passed to agents for processing
///
/// Each agent receives the same [`A2AManifest`] but can include additional
/// context specific to their role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    /// The manifest being processed
    pub manifest: A2AManifest,

    /// Additional context specific to this agent
    #[serde(default)]
    pub context: HashMap<String, serde_json::Value>,
}

impl AgentInput {
    /// Creates a new agent input from a manifest
    pub fn new(manifest: A2AManifest) -> Self {
        Self {
            manifest,
            context: HashMap::new(),
        }
    }

    /// Adds a key-value pair to the context
    pub fn with_context(mut self, key: &str, value: serde_json::Value) -> Self {
        self.context.insert(key.to_string(), value);
        self
    }
}

/// Output produced by an agent during processing
///
/// Contains the agent's response along with metadata for consensus evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    /// The agent's name
    pub agent: String,

    /// The generated content
    pub content: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Reasoning/explanation (optional)
    pub reasoning: Option<String>,

    /// Tokens used
    pub tokens_used: u32,

    /// Processing time in milliseconds
    pub latency_ms: u64,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Consensus vote (for council voting)
    pub vote: Option<ConsensusVote>,
}

impl AgentOutput {
    /// Creates a new agent output with minimal fields
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent's name
    /// * `content` - The response content
    /// * `confidence` - Confidence score (0.0-1.0, will be clamped if outside range)
    pub fn new(agent: &str, content: String, confidence: f32) -> Self {
        // Clamp confidence to valid range [0.0, 1.0] to prevent invalid values
        let confidence = confidence.clamp(0.0, 1.0);

        Self {
            agent: agent.to_string(),
            content,
            confidence,
            reasoning: None,
            tokens_used: 0,
            latency_ms: 0,
            metadata: HashMap::new(),
            vote: None,
        }
    }

    /// Adds reasoning information to the output
    pub fn with_reasoning(mut self, reasoning: String) -> Self {
        self.reasoning = Some(reasoning);
        self
    }

    /// Sets the number of tokens consumed
    pub fn with_tokens(mut self, tokens: u32) -> Self {
        self.tokens_used = tokens;
        self
    }

    /// Sets the processing latency
    pub fn with_latency(mut self, ms: u64) -> Self {
        self.latency_ms = ms;
        self
    }

    /// Adds a consensus vote to the output
    pub fn with_vote(mut self, vote: ConsensusVote) -> Self {
        self.vote = Some(vote);
        self
    }
}

/// Vote from an agent in the consensus process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVote {
    /// Agent name
    pub agent: String,

    /// Approval decision
    pub approve: bool,

    /// Confidence in this vote (0.0-1.0)
    pub confidence: f32,

    /// Reasoning for the vote
    pub reasoning: Option<String>,

    /// Any concerns or constraints identified
    #[serde(default)]
    pub concerns: Vec<Constraint>,
}

impl ConsensusVote {
    /// Create a new vote
    pub fn new(agent: &str, approve: bool, confidence: f32) -> Self {
        Self {
            agent: agent.to_string(),
            approve,
            confidence,
            reasoning: None,
            concerns: vec![],
        }
    }

    /// Add reasoning
    pub fn with_reasoning(mut self, reasoning: String) -> Self {
        self.reasoning = Some(reasoning);
        self
    }

    /// Add concerns
    pub fn with_concerns(mut self, concerns: Vec<Constraint>) -> Self {
        self.concerns = concerns;
        self
    }
}

/// Verification constraint from an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Type of constraint
    pub constraint_type: ConstraintType,

    /// Severity level
    pub severity: Severity,

    /// Description of the issue
    pub description: String,

    /// Evidence for the constraint
    pub source: Option<String>,

    /// How to fix it
    pub suggestion: Option<String>,
}

/// Type of constraint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Factual accuracy
    Fact,
    /// Hardware limitations
    Hardware,
    /// Safety concern
    Safety,
    /// Code quality
    Quality,
}

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Just a warning
    Warning,
    /// Needs fixing
    Error,
    /// Critical - blocks execution
    Critical,
}

/// Configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Model identifier
    pub model: String,

    /// Whether this agent is enabled
    pub enabled: bool,

    /// Temperature for generation (0.0-1.0)
    pub temperature: f32,

    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// System prompt for this agent
    pub system_prompt: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            enabled: true,
            temperature: 0.7,
            max_tokens: 2048,
            system_prompt: None,
        }
    }
}

// ============================================================================
// Blanket Implementations for Smart Pointers
// ============================================================================

/// Blanket implementation for Arc<T> where T implements Agent
#[async_trait]
impl<T: Agent + ?Sized> Agent for std::sync::Arc<T> {
    fn name(&self) -> &str {
        self.as_ref().name()
    }

    fn role(&self) -> &str {
        self.as_ref().role()
    }

    async fn process(&self, input: AgentInput) -> Result<AgentOutput> {
        self.as_ref().process(input).await
    }

    fn is_ready(&self) -> bool {
        self.as_ref().is_ready()
    }

    fn model(&self) -> &str {
        self.as_ref().model()
    }
}

/// Blanket implementation for Box<T> where T implements Agent
#[async_trait]
impl<T: Agent + ?Sized> Agent for Box<T> {
    fn name(&self) -> &str {
        self.as_ref().name()
    }

    fn role(&self) -> &str {
        self.as_ref().role()
    }

    async fn process(&self, input: AgentInput) -> Result<AgentOutput> {
        self.as_ref().process(input).await
    }

    fn is_ready(&self) -> bool {
        self.as_ref().is_ready()
    }

    fn model(&self) -> &str {
        self.as_ref().model()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_output_builder() {
        let response = AgentOutput::new("test", "Hello".to_string(), 0.9)
            .with_reasoning("Because...".to_string())
            .with_tokens(100)
            .with_latency(50);

        assert_eq!(response.agent, "test");
        assert_eq!(response.confidence, 0.9);
        assert_eq!(response.tokens_used, 100);
        assert_eq!(response.latency_ms, 50);
    }

    #[test]
    fn test_consensus_vote() {
        let vote = ConsensusVote::new("agent1", true, 0.85)
            .with_reasoning("Clear intent".to_string())
            .with_concerns(vec![]);

        assert_eq!(vote.agent, "agent1");
        assert!(vote.approve);
        assert_eq!(vote.confidence, 0.85);
        assert!(vote.reasoning.is_some());
    }

    #[test]
    fn test_constraint_types() {
        let constraint = Constraint {
            constraint_type: ConstraintType::Safety,
            severity: Severity::Critical,
            description: "Dangerous operation".to_string(),
            source: None,
            suggestion: Some("Use alternative approach".to_string()),
        };

        assert_eq!(constraint.severity, Severity::Critical);
        assert!(constraint.suggestion.is_some());
    }

    #[test]
    fn test_agent_input_builder() {
        let manifest = A2AManifest::new("Test query".to_string());
        let input = AgentInput::new(manifest).with_context("key", serde_json::json!("value"));

        assert!(input.context.contains_key("key"));
    }

    #[test]
    fn test_confidence_clamping() {
        // Test negative confidence (should clamp to 0.0)
        let output1 = AgentOutput::new("test", "content".to_string(), -0.5);
        assert_eq!(output1.confidence, 0.0);

        // Test confidence > 1.0 (should clamp to 1.0)
        let output2 = AgentOutput::new("test", "content".to_string(), 1.5);
        assert_eq!(output2.confidence, 1.0);

        // Test normal confidence (should remain unchanged)
        let output3 = AgentOutput::new("test", "content".to_string(), 0.85);
        assert_eq!(output3.confidence, 0.85);

        // Test boundary values
        let output4 = AgentOutput::new("test", "content".to_string(), 0.0);
        assert_eq!(output4.confidence, 0.0);

        let output5 = AgentOutput::new("test", "content".to_string(), 1.0);
        assert_eq!(output5.confidence, 1.0);
    }
}
