//! Multi-round revision example
//!
//! Demonstrates how agents improve their responses across rounds:
//! - Round 1: Low confidence (0.60)
//! - Round 2: High confidence (0.95) after receiving feedback
//! - Shows feedback mechanism and revision process

use async_trait::async_trait;
use std::sync::Arc;
use tripartite::{Agent, AgentInput, AgentOutput, AgentWeights, ConsensusConfig, ConsensusEngine};

/// Agent that needs multiple rounds to reach high confidence
struct RevisingAgent {
    name: String,
    round_needed: u8, // Number of rounds before reaching high confidence
}

#[async_trait]
impl Agent for RevisingAgent {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput, tripartite::Error> {
        let current_round = input.manifest.round;

        // Check for feedback from previous rounds
        let reasoning = if let Some(feedback) = input.manifest.feedback.last() {
            format!("Received feedback: {}", feedback)
        } else {
            "First attempt".to_string()
        };

        // Low confidence in early rounds, high in later rounds
        let confidence = if current_round >= self.round_needed {
            0.95 // High confidence after enough rounds
        } else {
            0.60 // Low confidence in early rounds
        };

        Ok(AgentOutput::new(
            &self.name,
            format!("Response from round {}", current_round),
            confidence,
        )
        .with_reasoning(reasoning))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn role(&self) -> &str {
        "Revising agent that improves with feedback"
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model(&self) -> &str {
        "revising-model"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("=== Multi-Round Revision Example ===\n");

    // Create three agents that all need 2 rounds to converge
    let agent_1 = Arc::new(RevisingAgent {
        name: "Agent 1".to_string(),
        round_needed: 2,
    });

    let agent_2 = Arc::new(RevisingAgent {
        name: "Agent 2".to_string(),
        round_needed: 2,
    });

    let agent_3 = Arc::new(RevisingAgent {
        name: "Agent 3".to_string(),
        round_needed: 2,
    });

    println!("Created 3 revising agents:");
    println!(
        "  - Agent 1 (needs {} rounds to converge)",
        agent_1.round_needed
    );
    println!(
        "  - Agent 2 (needs {} rounds to converge)",
        agent_2.round_needed
    );
    println!(
        "  - Agent 3 (needs {} rounds to converge)",
        agent_3.round_needed
    );
    println!();

    // Configure consensus with uniform weights
    let config = ConsensusConfig {
        threshold: 0.85,                  // 85% threshold
        max_rounds: 5,                    // Allow up to 5 rounds
        weights: AgentWeights::default(), // Default weights
    };

    println!("Consensus Configuration:");
    println!("  - Threshold: {:.0}", config.threshold * 100.0);
    println!("  - Max Rounds: {}", config.max_rounds);
    println!("  - Weights: Default");
    println!();

    let mut engine = ConsensusEngine::new(config, agent_1, agent_2, agent_3);

    println!("Running consensus: \"Complex query requiring analysis\"\n");

    let outcome = engine.run("Complex query requiring analysis").await?;

    println!("=== Results ===");
    println!("Consensus Reached: {}", outcome.is_consensus());
    println!("Rounds Needed: {}", outcome.rounds());
    println!(
        "Final Confidence: {:.2}",
        outcome.aggregate_confidence().unwrap_or(0.0)
    );
    println!("Duration: {}ms", outcome.total_duration_ms);

    // Show progression across rounds
    println!("\n=== Round Progression ===");
    println!("Round 1:");
    println!("  - All agents: 0.60 confidence");
    println!("  - Aggregate: 0.60 (below 0.85 threshold)");
    println!("  - Action: Generate feedback, proceed to Round 2");

    println!("\nRound 2:");
    println!("  - All agents: 0.95 confidence (after receiving feedback)");
    println!("  - Aggregate: 0.95 (above 0.85 threshold)");
    println!("  - Action: Consensus reached!");

    // Show feedback if available
    if let Some(ref reasoning) = outcome.reasoning {
        println!("\nReasoning: {}", reasoning);
    }

    println!("\n=== Why Multi-Round Matters ===");
    println!("Some agents need time to:");
    println!("  1. Understand the full context");
    println!("  2. Retrieve relevant information");
    println!("  3. Process feedback from other agents");
    println!("  4. Refine their responses");
    println!();
    println!("Multi-round consensus allows agents to iteratively");
    println!("improve their answers until they reach high confidence.");

    Ok(())
}
