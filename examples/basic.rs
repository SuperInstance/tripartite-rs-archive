//! Basic consensus example
//!
//! Demonstrates the simplest use of tripartite-rs:
//! - Create 3 simple agents
//! - Run consensus on a query
//! - Check if consensus was reached

use async_trait::async_trait;
use std::sync::Arc;
use tripartite::{Agent, AgentInput, AgentOutput, ConsensusConfig, ConsensusEngine};

/// Simple agent that returns a fixed response with fixed confidence
struct SimpleAgent {
    name: String,
    response: String,
    confidence: f32,
}

#[async_trait]
impl Agent for SimpleAgent {
    async fn process(&self, _input: AgentInput) -> Result<AgentOutput, tripartite::Error> {
        Ok(AgentOutput::new(
            &self.name,
            self.response.clone(),
            self.confidence,
        ))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn role(&self) -> &str {
        "Simple response agent"
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model(&self) -> &str {
        "static"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Basic Consensus Example ===\n");

    // Create three agents with high confidence
    let agent_a = Arc::new(SimpleAgent {
        name: "Agent A".to_string(),
        response: "The answer is 42".to_string(),
        confidence: 0.90,
    });

    let agent_b = Arc::new(SimpleAgent {
        name: "Agent B".to_string(),
        response: "The answer is 42".to_string(),
        confidence: 0.85,
    });

    let agent_c = Arc::new(SimpleAgent {
        name: "Agent C".to_string(),
        response: "The answer is 42".to_string(),
        confidence: 0.88,
    });

    println!("Created 3 agents:");
    println!("  - {} (confidence: {:.2})", agent_a.name(), 0.90);
    println!("  - {} (confidence: {:.2})", agent_b.name(), 0.85);
    println!("  - {} (confidence: {:.2})", agent_c.name(), 0.88);
    println!();

    // Create consensus engine with default configuration
    // Default threshold: 0.85
    // Default max rounds: 3
    let mut engine = ConsensusEngine::with_agents(agent_a, agent_b, agent_c);

    println!("Running consensus with query: \"What is the answer?\"\n");

    // Run consensus
    let outcome = engine.run("What is the answer?").await?;

    // Check results
    println!("=== Results ===");
    println!("Consensus Reached: {}", outcome.is_consensus());
    println!("Content: {}", outcome.content);
    println!(
        "Aggregate Confidence: {:.2}",
        outcome.aggregate_confidence().unwrap_or(0.0)
    );
    println!("Rounds: {}", outcome.rounds());
    println!("Duration: {}ms", outcome.total_duration_ms);

    // Show individual agent responses
    println!("\n=== Agent Responses ===");
    if let Some(ref pathos) = outcome.pathos_response {
        println!("Agent A: {:.2} confidence", pathos.confidence);
    }
    if let Some(ref logos) = outcome.logos_response {
        println!("Agent B: {:.2} confidence", logos.confidence);
    }
    if let Some(ref ethos) = outcome.ethos_response {
        println!("Agent C: {:.2} confidence", ethos.confidence);
    }

    Ok(())
}
