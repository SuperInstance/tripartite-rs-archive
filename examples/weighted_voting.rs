//! Weighted voting example
//!
//! Demonstrates how to configure agent weights:
//! - Senior expert gets 50% weight
//! - Mid-level expert gets 30% weight
//! - Junior expert gets 20% weight
//! - Consensus threshold: 0.90

use async_trait::async_trait;
use std::sync::Arc;
use tripartite::{Agent, AgentInput, AgentOutput, AgentWeights, ConsensusConfig, ConsensusEngine};

/// Expert agent with a fixed response
struct ExpertAgent {
    name: String,
    confidence: f32,
}

#[async_trait]
impl Agent for ExpertAgent {
    async fn process(&self, _input: AgentInput) -> Result<AgentOutput, tripartite::Error> {
        Ok(AgentOutput::new(
            &self.name,
            "Analysis: Deploy to production".to_string(),
            self.confidence,
        ))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn role(&self) -> &str {
        "Expert analyst"
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model(&self) -> &str {
        "expert-model"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("=== Weighted Voting Example ===\n");

    // Create three experts with different confidence levels
    let senior = Arc::new(ExpertAgent {
        name: "Senior Engineer".to_string(),
        confidence: 0.95, // Very confident
    });

    let mid_level = Arc::new(ExpertAgent {
        name: "Mid-level Engineer".to_string(),
        confidence: 0.75, // Moderately confident
    });

    let junior = Arc::new(ExpertAgent {
        name: "Junior Engineer".to_string(),
        confidence: 0.60, // Less confident
    });

    println!("Created 3 experts:");
    println!(
        "  - Senior Engineer: {:.2} confidence (50% weight)",
        senior.confidence
    );
    println!(
        "  - Mid-level Engineer: {:.2} confidence (30% weight)",
        mid_level.confidence
    );
    println!(
        "  - Junior Engineer: {:.2} confidence (20% weight)",
        junior.confidence
    );
    println!();

    // Configure consensus with custom weights
    let config = ConsensusConfig {
        threshold: 0.90, // Higher threshold (90%)
        max_rounds: 3,
        weights: AgentWeights {
            pathos: 0.50, // Senior gets 50% weight
            logos: 0.30,  // Mid-level gets 30% weight
            ethos: 0.20,  // Junior gets 20% weight
        },
    };

    println!("Consensus Configuration:");
    println!("  - Threshold: {:.0}", config.threshold * 100.0);
    println!("  - Max Rounds: {}", config.max_rounds);
    println!(
        "  - Agent 1 (Senior) Weight: {:.0}",
        config.weights.pathos * 100.0
    );
    println!(
        "  - Agent 2 (Mid) Weight: {:.0}",
        config.weights.logos * 100.0
    );
    println!(
        "  - Agent 3 (Junior) Weight: {:.0}",
        config.weights.ethos * 100.0
    );
    println!();

    // Create engine and run consensus
    let mut engine = ConsensusEngine::new(config, senior, mid_level, junior);

    println!("Running consensus: \"Should we deploy to production?\"\n");

    let outcome = engine.run("Should we deploy to production?").await?;

    // Calculate weighted aggregate manually for demonstration
    // (0.95 * 0.50) + (0.75 * 0.30) + (0.60 * 0.20)
    // = 0.475 + 0.225 + 0.12
    // = 0.82
    let expected_aggregate = 0.82;

    println!("=== Results ===");
    println!("Consensus Reached: {}", outcome.is_consensus());
    println!(
        "Aggregate Confidence: {:.2}",
        outcome.aggregate_confidence().unwrap_or(0.0)
    );
    println!(
        "Expected Aggregate: {:.2} (calculated manually)",
        expected_aggregate
    );
    println!("Threshold Required: {:.2}", 0.90);
    println!("Rounds: {}", outcome.rounds());

    if outcome.is_consensus() {
        println!("\n✓ Consensus reached! Decision: {}", outcome.content);
    } else {
        println!("\n✗ Consensus not reached. Need more discussion.");
        if let Some(ref reasoning) = outcome.reasoning {
            println!("Reason: {}", reasoning);
        }
    }

    println!("\n=== Why This Matters ===");
    println!("Even though two experts are below 90%, the senior expert's");
    println!("high confidence (95%) carries 50% weight, which significantly");
    println!("influences the final decision. However, in this case, the");
    println!("aggregate (82%) still falls short of the 90% threshold.");

    Ok(())
}
