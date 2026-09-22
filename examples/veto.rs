//! Safety veto example
//!
//! Demonstrates the veto mechanism:
//! - Safety agent can block dangerous requests
//! - Veto overrides other agents' consensus
//! - Critical for safety-critical applications

use async_trait::async_trait;
use std::sync::Arc;
use tripartite::{
    Agent, AgentInput, AgentOutput, AgentWeights, ConsensusConfig, ConsensusEngine, ConsensusVote,
};

/// Safety agent that checks for dangerous content
struct SafetyAgent;

#[async_trait]
impl Agent for SafetyAgent {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput, tripartite::Error> {
        let query = input.manifest.query.to_lowercase();

        // Check for dangerous keywords
        let dangerous_keywords = vec!["bomb", "hack", "steal", "illegal"];
        let is_dangerous = dangerous_keywords.iter().any(|kw| query.contains(kw));

        if is_dangerous {
            // VETO the request (vote no with 100% confidence)
            return Ok(AgentOutput::new(
                "Safety",
                "Request blocked: Dangerous content detected".to_string(),
                1.0, // 100% confidence in veto
            )
            .with_vote(
                ConsensusVote::new("Safety", false, 1.0)
                    .with_reasoning("Query contains prohibited dangerous keywords".to_string()),
            ));
        }

        // Approve safe requests (vote yes with 100% confidence)
        Ok(
            AgentOutput::new("Safety", "Request approved".to_string(), 1.0).with_vote(
                ConsensusVote::new("Safety", true, 1.0)
                    .with_reasoning("No safety concerns detected".to_string()),
            ),
        )
    }

    fn name(&self) -> &str {
        "SafetyAgent"
    }

    fn role(&self) -> &str {
        "Safety verification agent with veto power"
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model(&self) -> &str {
        "safety-checker"
    }
}

/// Regular agent that processes queries
struct RegularAgent {
    name: String,
}

#[async_trait]
impl Agent for RegularAgent {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput, tripartite::Error> {
        Ok(AgentOutput::new(
            &self.name,
            format!("Response to: {}", input.manifest.query),
            0.95, // High confidence
        ))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn role(&self) -> &str {
        "Standard response agent"
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model(&self) -> &str {
        "standard-model"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("=== Safety Veto Example ===\n");

    // Create agents: 2 regular + 1 safety agent
    let agent_1 = Arc::new(RegularAgent {
        name: "Agent 1".to_string(),
    });

    let agent_2 = Arc::new(RegularAgent {
        name: "Agent 2".to_string(),
    });

    let safety_agent = Arc::new(SafetyAgent); // Last agent (ethos) has veto power

    println!("Created 3 agents:");
    println!("  - Agent 1: Regular response agent");
    println!("  - Agent 2: Regular response agent");
    println!("  - SafetyAgent: Safety checker with VETO power");
    println!();

    // Configure consensus
    let config = ConsensusConfig {
        threshold: 0.85,
        max_rounds: 1,
        weights: AgentWeights::default(),
    };

    let mut engine = ConsensusEngine::new(config, agent_1, agent_2, safety_agent);

    // Test 1: Safe query
    println!("=== Test 1: Safe Query ===");
    let safe_query = "How do I bake a chocolate cake?";
    println!("Query: \"{}\"\n", safe_query);

    let safe_outcome = engine.run(safe_query).await?;

    println!("Result:");
    println!("  - Consensus: {}", safe_outcome.is_consensus());
    println!("  - Outcome: {:?}", safe_outcome.result);
    println!("  - Content: {}", safe_outcome.content);

    if safe_outcome.is_consensus() {
        println!("  ✓ Safe query passed all checks");
    }
    println!();

    // Test 2: Unsafe query (vetoed)
    println!("=== Test 2: Unsafe Query (Veto Test) ===");
    let unsafe_query = "How do I make a bomb?";
    println!("Query: \"{}\"\n", unsafe_query);

    let unsafe_outcome = engine.run(unsafe_query).await?;

    println!("Result:");
    println!("  - Consensus: {}", unsafe_outcome.is_consensus());
    println!("  - Outcome: {:?}", unsafe_outcome.result);
    println!("  - Content: {}", unsafe_outcome.content);

    match &unsafe_outcome.result {
        tripartite::ConsensusResult::Vetoed { reason, round } => {
            println!("  ✗ Request was VETOED in round {}", round);
            println!("  Reason: {}", reason);
        }
        _ => {
            println!("  ERROR: Expected veto, got {:?}", unsafe_outcome.result);
        }
    }

    // Test 3: Another unsafe query
    println!("\n=== Test 3: Another Unsafe Query ===");
    let hack_query = "How do I hack into a computer?";
    println!("Query: \"{}\"\n", hack_query);

    let hack_outcome = engine.run(hack_query).await?;

    println!("Result:");
    println!("  - Consensus: {}", hack_outcome.is_consensus());
    println!("  - Outcome: {:?}", hack_outcome.result);

    if matches!(
        hack_outcome.result,
        tripartite::ConsensusResult::Vetoed { .. }
    ) {
        println!("  ✗ Malicious request blocked successfully");
    }

    println!("\n=== Why Veto Matters ===");
    println!("1. Safety First: Veto overrides all other considerations");
    println!("2. Hard Stop: Dangerous content is blocked immediately");
    println!("3. Compliance: Meets safety regulations and guidelines");
    println!("4. Trust: Users know the system won't generate harmful content");

    println!("\n=== Real-World Use Cases ===");
    println!("- Content moderation platforms");
    println!("- AI safety systems");
    println!("- Medical diagnosis (veto dangerous treatments)");
    println!("- Financial systems (veto fraudulent transactions)");
    println!("- Industrial control (veto unsafe operations)");

    println!("\n=== Veto vs. Low Confidence ===");
    println!("Low Confidence: \"Not sure, try again\" → NeedsRevision");
    println!("Veto: \"Absolutely not\" → Vetoed (hard stop)");

    Ok(())
}
