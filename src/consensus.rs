//! Consensus engine for coordinating multiple agents
//!
//! This module implements the consensus mechanism that coordinates agents
//! to reach agreement on responses before they are returned.

use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{debug, info, instrument, warn};

use crate::agent::{Agent, AgentInput, AgentOutput};
use crate::error::Result;
use crate::manifest::A2AManifest;

/// Consensus engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Minimum confidence threshold for consensus (0.0-1.0)
    pub threshold: f32,

    /// Maximum rounds before giving up
    pub max_rounds: u8,

    /// Weights for each agent's vote
    pub weights: AgentWeights,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            threshold: 0.85,
            max_rounds: 3,
            weights: AgentWeights::default(),
        }
    }
}

/// Voting weights for each agent
///
/// The weights determine how much each agent's confidence contributes
/// to the final aggregate score. Weights should sum to approximately 1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentWeights {
    /// Weight for the first agent (e.g., Pathos - Intent)
    pub pathos: f32,

    /// Weight for the second agent (e.g., Logos - Logic)
    pub logos: f32,

    /// Weight for the third agent (e.g., Ethos - Truth)
    pub ethos: f32,
}

impl Default for AgentWeights {
    fn default() -> Self {
        Self {
            pathos: 0.25,
            logos: 0.45,
            ethos: 0.30,
        }
    }
}

/// The consensus engine
///
/// This is the main orchestrator that runs agents in sequence and evaluates
/// their responses to reach consensus.
///
/// # Type Parameters
///
/// * `P` - First agent type (e.g., Pathos - Intent extraction)
/// * `L` - Second agent type (e.g., Logos - Logic synthesis)
/// * `E` - Third agent type (e.g., Ethos - Truth verification)
pub struct ConsensusEngine<P, L, E>
where
    P: Agent,
    L: Agent,
    E: Agent,
{
    config: ConsensusConfig,
    pathos: P,
    logos: L,
    ethos: E,
}

impl<P, L, E> ConsensusEngine<P, L, E>
where
    P: Agent,
    L: Agent,
    E: Agent,
{
    /// Create a new consensus engine
    pub fn new(config: ConsensusConfig, pathos: P, logos: L, ethos: E) -> Self {
        Self {
            config,
            pathos,
            logos,
            ethos,
        }
    }

    /// Create a consensus engine with default configuration
    pub fn with_agents(pathos: P, logos: L, ethos: E) -> Self {
        Self::new(ConsensusConfig::default(), pathos, logos, ethos)
    }

    /// Run the consensus process for a given prompt
    #[instrument(skip(self, prompt))]
    pub async fn run(&mut self, prompt: &str) -> Result<ConsensusOutcome> {
        let start_time = Instant::now();
        info!("Starting consensus process for prompt: {}", prompt);

        // Create initial manifest
        let mut manifest = A2AManifest::new(prompt.to_string());

        // Run consensus rounds
        for round in 1..=self.config.max_rounds {
            debug!("=== Starting Round {} ===", round);

            // Run first agent
            let pathos_start = Instant::now();
            let pathos_input = AgentInput {
                manifest: manifest.clone(),
                context: std::collections::HashMap::new(),
            };
            let pathos_response = self.pathos.process(pathos_input).await?;
            let pathos_duration = pathos_start.elapsed();

            debug!(
                "Agent 1 ({}) completed in {:?} (confidence: {:.2})",
                self.pathos.name(),
                pathos_duration,
                pathos_response.confidence
            );

            // Update manifest with first agent framing
            manifest.pathos_framing = Some(pathos_response.content.clone());
            manifest.pathos_confidence = Some(pathos_response.confidence);

            // Run second agent
            let logos_start = Instant::now();
            let logos_input = AgentInput {
                manifest: manifest.clone(),
                context: std::collections::HashMap::new(),
            };
            let logos_response = self.logos.process(logos_input).await?;
            let logos_duration = logos_start.elapsed();

            debug!(
                "Agent 2 ({}) completed in {:?} (confidence: {:.2})",
                self.logos.name(),
                logos_duration,
                logos_response.confidence
            );

            // Run third agent
            let ethos_start = Instant::now();
            let ethos_input = AgentInput {
                manifest: manifest.clone(),
                context: std::collections::HashMap::new(),
            };
            let ethos_response = self.ethos.process(ethos_input).await?;
            let ethos_duration = ethos_start.elapsed();

            debug!(
                "Agent 3 ({}) completed in {:?} (confidence: {:.2})",
                self.ethos.name(),
                ethos_duration,
                ethos_response.confidence
            );

            // Evaluate consensus
            let result = self.evaluate(&pathos_response, &logos_response, &ethos_response, round);

            match result {
                ConsensusResult::Reached {
                    aggregate_confidence,
                    ..
                } => {
                    info!(
                        "Consensus reached in round {} with confidence {:.2}",
                        round, aggregate_confidence
                    );

                    return Ok(ConsensusOutcome {
                        result,
                        content: logos_response.content.clone(),
                        reasoning: logos_response.reasoning.clone(),
                        pathos_response: Some(pathos_response),
                        logos_response: Some(logos_response),
                        ethos_response: Some(ethos_response),
                        total_duration_ms: start_time.elapsed().as_millis() as u64,
                    });
                }
                ConsensusResult::Vetoed { ref reason, .. } => {
                    warn!("Agent 3 vetoed the response: {}", reason);
                    let reason_text = reason.clone();

                    return Ok(ConsensusOutcome {
                        result,
                        content: reason_text,
                        reasoning: Some("Response vetoed by third agent".to_string()),
                        pathos_response: Some(pathos_response),
                        logos_response: Some(logos_response),
                        ethos_response: Some(ethos_response),
                        total_duration_ms: start_time.elapsed().as_millis() as u64,
                    });
                }
                ConsensusResult::NotReached {
                    aggregate_confidence,
                    rounds_attempted,
                    ..
                } => {
                    warn!(
                        "Consensus not reached after {} rounds (final confidence: {:.2})",
                        rounds_attempted, aggregate_confidence
                    );

                    return Ok(ConsensusOutcome {
                        result,
                        content: logos_response.content.clone(),
                        reasoning: Some(format!(
                            "Consensus not reached after {} rounds",
                            rounds_attempted
                        )),
                        pathos_response: Some(pathos_response),
                        logos_response: Some(logos_response),
                        ethos_response: Some(ethos_response),
                        total_duration_ms: start_time.elapsed().as_millis() as u64,
                    });
                }
                ConsensusResult::NeedsRevision {
                    feedback, round, ..
                } => {
                    info!(
                        "Round {} incomplete: {}. Retry with feedback...",
                        round, feedback
                    );

                    // Inject feedback into manifest for next round
                    manifest.add_feedback(feedback);
                    manifest.next_round();

                    continue;
                }
            }
        }

        // Should not reach here due to max_rounds check
        unreachable!()
    }

    /// Calculate weighted aggregate confidence
    pub fn calculate_aggregate(&self, pathos: f32, logos: f32, ethos: f32) -> f32 {
        let weights = &self.config.weights;

        (pathos * weights.pathos) + (logos * weights.logos) + (ethos * weights.ethos)
    }

    /// Evaluate responses and determine if consensus is reached
    #[instrument(skip(self, pathos, logos, ethos))]
    pub fn evaluate(
        &self,
        pathos: &AgentOutput,
        logos: &AgentOutput,
        ethos: &AgentOutput,
        round: u8,
    ) -> ConsensusResult {
        debug!(
            "Evaluating consensus - Round {}: Agent1={:.2}, Agent2={:.2}, Agent3={:.2}",
            round, pathos.confidence, logos.confidence, ethos.confidence
        );

        // Check for third agent veto
        if let Some(verdict) = ethos.metadata.get("verdict") {
            if let Ok(v) = serde_json::from_value::<Verdict>(verdict.clone()) {
                if v == Verdict::Veto {
                    warn!("Third agent vetoed the response");
                    return ConsensusResult::Vetoed {
                        reason: ethos.content.clone(),
                        round,
                    };
                }
            }
        }

        // Calculate aggregate confidence
        let aggregate =
            self.calculate_aggregate(pathos.confidence, logos.confidence, ethos.confidence);

        info!(
            "Aggregate confidence: {:.2} (threshold: {:.2})",
            aggregate, self.config.threshold
        );

        // Check if consensus is reached
        if aggregate >= self.config.threshold {
            ConsensusResult::Reached {
                aggregate_confidence: aggregate,
                round,
                votes: Votes {
                    pathos: pathos.confidence,
                    logos: logos.confidence,
                    ethos: ethos.confidence,
                },
            }
        } else if round >= self.config.max_rounds {
            // Max rounds exceeded
            ConsensusResult::NotReached {
                aggregate_confidence: aggregate,
                rounds_attempted: round,
                votes: Votes {
                    pathos: pathos.confidence,
                    logos: logos.confidence,
                    ethos: ethos.confidence,
                },
            }
        } else {
            // Need another round
            ConsensusResult::NeedsRevision {
                aggregate_confidence: aggregate,
                round,
                feedback: self.generate_feedback(pathos, logos, ethos),
            }
        }
    }

    /// Generate feedback for the next round
    fn generate_feedback(
        &self,
        pathos: &AgentOutput,
        logos: &AgentOutput,
        ethos: &AgentOutput,
    ) -> String {
        let mut feedback = String::new();

        // Identify the lowest confidence agent
        let min_confidence = pathos
            .confidence
            .min(logos.confidence)
            .min(ethos.confidence);

        if pathos.confidence == min_confidence {
            feedback.push_str(&format!(
                "{} is uncertain about the intent. ",
                self.pathos.name()
            ));
            if let Some(reasoning) = &pathos.reasoning {
                feedback.push_str(&format!("{} says: {}. ", self.pathos.name(), reasoning));
            }
        }

        if logos.confidence == min_confidence {
            feedback.push_str(&format!(
                "{} needs more context or has concerns about the reasoning. ",
                self.logos.name()
            ));
            if let Some(reasoning) = &logos.reasoning {
                feedback.push_str(&format!("{} says: {}. ", self.logos.name(), reasoning));
            }
        }

        if ethos.confidence == min_confidence {
            feedback.push_str(&format!(
                "{} has concerns about the response. ",
                self.ethos.name()
            ));
            if let Some(reasoning) = &ethos.reasoning {
                feedback.push_str(&format!("{} says: {}. ", self.ethos.name(), reasoning));
            }
        }

        feedback
    }

    /// Get the threshold
    pub fn threshold(&self) -> f32 {
        self.config.threshold
    }

    /// Get max rounds
    pub fn max_rounds(&self) -> u8 {
        self.config.max_rounds
    }
}

/// Verdict from the third agent (verifier)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// Response approved
    Approved,
    /// Response needs revision
    NeedsRevision,
    /// Response vetoed (hard stop)
    Veto,
}

/// Result of consensus evaluation
#[derive(Debug, Clone)]
pub enum ConsensusResult {
    /// Consensus reached
    Reached {
        aggregate_confidence: f32,
        round: u8,
        votes: Votes,
    },
    /// Consensus not reached after max rounds
    NotReached {
        aggregate_confidence: f32,
        rounds_attempted: u8,
        votes: Votes,
    },
    /// Needs another round with feedback
    NeedsRevision {
        aggregate_confidence: f32,
        round: u8,
        feedback: String,
    },
    /// Third agent vetoed the response
    Vetoed { reason: String, round: u8 },
}

impl ConsensusResult {
    /// Check if consensus was reached
    pub fn is_consensus(&self) -> bool {
        matches!(self, ConsensusResult::Reached { .. })
    }

    /// Get the aggregate confidence if available
    pub fn aggregate_confidence(&self) -> Option<f32> {
        match self {
            ConsensusResult::Reached {
                aggregate_confidence,
                ..
            } => Some(*aggregate_confidence),
            ConsensusResult::NotReached {
                aggregate_confidence,
                ..
            } => Some(*aggregate_confidence),
            ConsensusResult::NeedsRevision {
                aggregate_confidence,
                ..
            } => Some(*aggregate_confidence),
            ConsensusResult::Vetoed { .. } => None,
        }
    }

    /// Get the round number
    pub fn round(&self) -> u8 {
        match self {
            ConsensusResult::Reached { round, .. } => *round,
            ConsensusResult::NotReached {
                rounds_attempted, ..
            } => *rounds_attempted,
            ConsensusResult::NeedsRevision { round, .. } => *round,
            ConsensusResult::Vetoed { round, .. } => *round,
        }
    }
}

/// Individual agent votes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Votes {
    pub pathos: f32,
    pub logos: f32,
    pub ethos: f32,
}

/// Complete outcome from a consensus run
#[derive(Debug, Clone)]
pub struct ConsensusOutcome {
    /// The consensus result (reached, not reached, vetoed)
    pub result: ConsensusResult,

    /// The final response content
    pub content: String,

    /// Optional reasoning/explanation
    pub reasoning: Option<String>,

    /// First agent response
    pub pathos_response: Option<AgentOutput>,

    /// Second agent response
    pub logos_response: Option<AgentOutput>,

    /// Third agent response
    pub ethos_response: Option<AgentOutput>,

    /// Total time for all rounds in milliseconds
    pub total_duration_ms: u64,
}

impl ConsensusOutcome {
    /// Check if consensus was reached
    pub fn is_consensus(&self) -> bool {
        self.result.is_consensus()
    }

    /// Get the aggregate confidence if available
    pub fn aggregate_confidence(&self) -> Option<f32> {
        self.result.aggregate_confidence()
    }

    /// Get the number of rounds
    pub fn rounds(&self) -> u8 {
        self.result.round()
    }

    /// Get a formatted summary
    pub fn summary(&self) -> String {
        format!(
            "Consensus: {} | Confidence: {:.2} | Rounds: {} | Duration: {}ms",
            if self.is_consensus() {
                "✓ REACHED"
            } else {
                "✗ NOT REACHED"
            },
            self.aggregate_confidence().unwrap_or(0.0),
            self.rounds(),
            self.total_duration_ms
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // =========================================================================
    // Test Fixtures
    // =========================================================================

    struct MockAgent {
        name: &'static str,
        role: &'static str,
        model: &'static str,
        confidence: f32,
    }

    #[async_trait::async_trait]
    impl Agent for MockAgent {
        fn name(&self) -> &str {
            self.name
        }

        fn role(&self) -> &str {
            self.role
        }

        async fn process(&self, _input: AgentInput) -> Result<AgentOutput> {
            Ok(AgentOutput::new(
                self.name,
                "test".to_string(),
                self.confidence,
            ))
        }

        fn is_ready(&self) -> bool {
            true
        }

        fn model(&self) -> &str {
            self.model
        }
    }

    fn mock_response(name: &str, confidence: f32) -> AgentOutput {
        AgentOutput {
            agent: name.to_string(),
            content: "test".to_string(),
            confidence,
            reasoning: None,
            tokens_used: 100,
            latency_ms: 10,
            metadata: HashMap::new(),
            vote: None,
        }
    }

    fn mock_response_with_reasoning(
        name: &str,
        confidence: f32,
        reasoning: Option<String>,
    ) -> AgentOutput {
        AgentOutput {
            agent: name.to_string(),
            content: "test".to_string(),
            confidence,
            reasoning,
            tokens_used: 100,
            latency_ms: 10,
            metadata: HashMap::new(),
            vote: None,
        }
    }

    fn mock_response_with_verdict(name: &str, confidence: f32, verdict: Verdict) -> AgentOutput {
        let mut metadata = HashMap::new();
        metadata.insert(
            "verdict".to_string(),
            serde_json::to_value(verdict).unwrap(),
        );

        AgentOutput {
            agent: name.to_string(),
            content: "test".to_string(),
            confidence,
            reasoning: None,
            tokens_used: 100,
            latency_ms: 10,
            metadata,
            vote: None,
        }
    }

    fn create_test_engine() -> ConsensusEngine<MockAgent, MockAgent, MockAgent> {
        let config = ConsensusConfig::default();
        let pathos = MockAgent {
            name: "Pathos",
            role: "Intent",
            model: "model1",
            confidence: 0.9,
        };
        let logos = MockAgent {
            name: "Logos",
            role: "Logic",
            model: "model2",
            confidence: 0.9,
        };
        let ethos = MockAgent {
            name: "Ethos",
            role: "Truth",
            model: "model3",
            confidence: 0.9,
        };
        ConsensusEngine::new(config, pathos, logos, ethos)
    }

    fn create_test_engine_with_threshold(
        threshold: f32,
    ) -> ConsensusEngine<MockAgent, MockAgent, MockAgent> {
        let config = ConsensusConfig {
            threshold,
            ..Default::default()
        };
        let pathos = MockAgent {
            name: "Pathos",
            role: "Intent",
            model: "model1",
            confidence: 0.9,
        };
        let logos = MockAgent {
            name: "Logos",
            role: "Logic",
            model: "model2",
            confidence: 0.9,
        };
        let ethos = MockAgent {
            name: "Ethos",
            role: "Truth",
            model: "model3",
            confidence: 0.9,
        };
        ConsensusEngine::new(config, pathos, logos, ethos)
    }

    // =========================================================================
    // Consensus Evaluation Tests
    // =========================================================================

    #[test]
    fn test_consensus_reached_high_confidence() {
        let engine = create_test_engine();

        let pathos = mock_response("Pathos", 0.95);
        let logos = mock_response("Logos", 0.92);
        let ethos = mock_response("Ethos", 0.88);

        let result = engine.evaluate(&pathos, &logos, &ethos, 1);
        assert!(result.is_consensus());
        assert!(matches!(result, ConsensusResult::Reached { .. }));

        if let ConsensusResult::Reached {
            aggregate_confidence,
            ..
        } = result
        {
            assert!(aggregate_confidence >= 0.85);
        }
    }

    #[test]
    fn test_consensus_reached_exact_threshold() {
        let engine = create_test_engine_with_threshold(0.90);

        // Values that should give exactly 0.90 with default weights
        let pathos = mock_response("Pathos", 0.90);
        let logos = mock_response("Logos", 0.90);
        let ethos = mock_response("Ethos", 0.90);

        let result = engine.evaluate(&pathos, &logos, &ethos, 1);
        assert!(result.is_consensus());
    }

    #[test]
    fn test_consensus_not_reached_low_confidence() {
        let engine = create_test_engine();

        let pathos = mock_response("Pathos", 0.6);
        let logos = mock_response("Logos", 0.5);
        let ethos = mock_response("Ethos", 0.4);

        let result = engine.evaluate(&pathos, &logos, &ethos, 3);
        assert!(!result.is_consensus());
        assert!(matches!(result, ConsensusResult::NotReached { .. }));
    }

    #[test]
    fn test_consensus_needs_revision() {
        let engine = create_test_engine();

        let pathos = mock_response("Pathos", 0.7);
        let logos = mock_response("Logos", 0.6);
        let ethos = mock_response("Ethos", 0.65);

        let result = engine.evaluate(&pathos, &logos, &ethos, 1);
        assert!(!result.is_consensus());
        assert!(matches!(result, ConsensusResult::NeedsRevision { .. }));

        if let ConsensusResult::NeedsRevision { feedback, .. } = result {
            assert!(!feedback.is_empty());
        }
    }

    #[test]
    fn test_consensus_vetoed_by_ethos() {
        let engine = create_test_engine();

        let pathos = mock_response("Pathos", 0.95);
        let logos = mock_response("Logos", 0.92);
        let ethos = mock_response_with_verdict("Ethos", 0.88, Verdict::Veto);

        let result = engine.evaluate(&pathos, &logos, &ethos, 1);
        assert!(!result.is_consensus());
        assert!(matches!(result, ConsensusResult::Vetoed { .. }));

        if let ConsensusResult::Vetoed { reason, round } = result {
            assert_eq!(reason, "test");
            assert_eq!(round, 1);
        }
    }

    #[test]
    fn test_consensus_max_rounds_exceeded() {
        let engine = create_test_engine();

        let pathos = mock_response("Pathos", 0.7);
        let logos = mock_response("Logos", 0.6);
        let ethos = mock_response("Ethos", 0.65);

        // At max rounds, should return NotReached instead of NeedsRevision
        let result = engine.evaluate(&pathos, &logos, &ethos, 3);
        assert!(!result.is_consensus());
        assert!(matches!(result, ConsensusResult::NotReached { .. }));
    }

    // =========================================================================
    // Aggregate Calculation Tests
    // =========================================================================

    #[test]
    fn test_aggregate_calculation_default_weights() {
        let engine = create_test_engine();

        // Default weights: pathos=0.25, logos=0.45, ethos=0.30
        // (1.0 * 0.25) + (1.0 * 0.45) + (1.0 * 0.30) = 1.0
        let aggregate = engine.calculate_aggregate(1.0, 1.0, 1.0);
        assert!((aggregate - 1.0).abs() < 0.001);

        // (0.0 * 0.25) + (0.0 * 0.45) + (0.0 * 0.30) = 0.0
        let aggregate = engine.calculate_aggregate(0.0, 0.0, 0.0);
        assert!((aggregate - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_aggregate_calculation_partial_confidence() {
        let engine = create_test_engine();

        // (0.8 * 0.25) + (0.9 * 0.45) + (0.7 * 0.30)
        // = 0.2 + 0.405 + 0.21 = 0.815
        let aggregate = engine.calculate_aggregate(0.8, 0.9, 0.7);
        assert!((aggregate - 0.815).abs() < 0.001);
    }

    #[test]
    fn test_aggregate_bounds() {
        let engine = create_test_engine();

        // Test that aggregate is always bounded [0, 1] with various confidence combinations
        let test_cases = vec![
            (0.0, 0.0, 0.0),
            (1.0, 1.0, 1.0),
            (0.5, 0.5, 0.5),
            (0.1, 0.9, 0.5),
            (0.3, 0.7, 0.2),
            (0.8, 0.1, 0.6),
            (0.99, 0.01, 0.5),
            (0.0, 1.0, 0.5),
            (1.0, 0.0, 0.0),
        ];

        for (p, l, e) in test_cases {
            let aggregate = engine.calculate_aggregate(p, l, e);
            assert!(
                (0.0..=1.0).contains(&aggregate),
                "Aggregate {} out of bounds for inputs ({}, {}, {})",
                aggregate,
                p,
                l,
                e
            );
        }
    }

    // =========================================================================
    // Feedback Generation Tests
    // =========================================================================

    #[test]
    fn test_feedback_identifies_lowest_confidence() {
        let engine = create_test_engine();

        let pathos = mock_response("Pathos", 0.5);
        let logos = mock_response("Logos", 0.8);
        let ethos = mock_response("Ethos", 0.9);

        let feedback = engine.generate_feedback(&pathos, &logos, &ethos);
        assert!(feedback.contains("Pathos"));
    }

    #[test]
    fn test_feedback_includes_reasoning() {
        let engine = create_test_engine();

        let pathos = mock_response_with_reasoning(
            "Pathos",
            0.5,
            Some("The user's intent is unclear".to_string()),
        );
        let logos = mock_response("Logos", 0.8);
        let ethos = mock_response("Ethos", 0.9);

        let feedback = engine.generate_feedback(&pathos, &logos, &ethos);
        assert!(feedback.contains("intent is unclear"));
    }

    #[test]
    fn test_feedback_multiple_low_confidence() {
        let engine = create_test_engine();

        let pathos = mock_response("Pathos", 0.5);
        let logos = mock_response("Logos", 0.5);
        let ethos = mock_response("Ethos", 0.9);

        let feedback = engine.generate_feedback(&pathos, &logos, &ethos);
        // Both Pathos and Logos are tied for lowest
        assert!(feedback.contains("Pathos") || feedback.contains("Logos"));
    }

    // =========================================================================
    // ConsensusResult Methods Tests
    // =========================================================================

    #[test]
    fn test_consensus_result_is_consensus() {
        let result = ConsensusResult::Reached {
            aggregate_confidence: 0.90,
            round: 1,
            votes: Votes {
                pathos: 0.9,
                logos: 0.9,
                ethos: 0.9,
            },
        };
        assert!(result.is_consensus());

        let result = ConsensusResult::NotReached {
            aggregate_confidence: 0.70,
            rounds_attempted: 3,
            votes: Votes {
                pathos: 0.7,
                logos: 0.7,
                ethos: 0.7,
            },
        };
        assert!(!result.is_consensus());

        let result = ConsensusResult::Vetoed {
            reason: "Unsafe".to_string(),
            round: 1,
        };
        assert!(!result.is_consensus());
    }

    #[test]
    fn test_consensus_result_aggregate_confidence() {
        let result = ConsensusResult::Reached {
            aggregate_confidence: 0.90,
            round: 1,
            votes: Votes {
                pathos: 0.9,
                logos: 0.9,
                ethos: 0.9,
            },
        };
        assert_eq!(result.aggregate_confidence(), Some(0.90));

        let result = ConsensusResult::Vetoed {
            reason: "Unsafe".to_string(),
            round: 1,
        };
        assert_eq!(result.aggregate_confidence(), None);
    }

    #[test]
    fn test_consensus_result_round() {
        let result = ConsensusResult::Reached {
            aggregate_confidence: 0.90,
            round: 2,
            votes: Votes {
                pathos: 0.9,
                logos: 0.9,
                ethos: 0.9,
            },
        };
        assert_eq!(result.round(), 2);

        let result = ConsensusResult::NotReached {
            aggregate_confidence: 0.70,
            rounds_attempted: 3,
            votes: Votes {
                pathos: 0.7,
                logos: 0.7,
                ethos: 0.7,
            },
        };
        assert_eq!(result.round(), 3);
    }

    // =========================================================================
    // Configuration Tests
    // =========================================================================

    #[test]
    fn test_config_default() {
        let config = ConsensusConfig::default();
        assert_eq!(config.threshold, 0.85);
        assert_eq!(config.max_rounds, 3);
        assert_eq!(config.weights.pathos, 0.25);
        assert_eq!(config.weights.logos, 0.45);
        assert_eq!(config.weights.ethos, 0.30);
    }

    #[test]
    fn test_weights_default() {
        let weights = AgentWeights::default();
        assert_eq!(weights.pathos, 0.25);
        assert_eq!(weights.logos, 0.45);
        assert_eq!(weights.ethos, 0.30);
    }

    #[test]
    fn test_weights_sum_to_one() {
        let weights = AgentWeights::default();
        let sum = weights.pathos + weights.logos + weights.ethos;
        assert!((sum - 1.0).abs() < 0.001);
    }

    // =========================================================================
    // Verdict Enum Tests
    // =========================================================================

    #[test]
    fn test_verdict_equality() {
        assert_eq!(Verdict::Approved, Verdict::Approved);
        assert_eq!(Verdict::NeedsRevision, Verdict::NeedsRevision);
        assert_eq!(Verdict::Veto, Verdict::Veto);

        assert_ne!(Verdict::Approved, Verdict::Veto);
        assert_ne!(Verdict::NeedsRevision, Verdict::Approved);
    }

    // =========================================================================
    // Edge Case Tests
    // =========================================================================

    #[test]
    fn test_boundary_confidence_values() {
        let engine = create_test_engine();

        // Test with confidence at exactly threshold
        let pathos = mock_response("Pathos", 0.85);
        let logos = mock_response("Logos", 0.85);
        let ethos = mock_response("Ethos", 0.85);

        let result = engine.evaluate(&pathos, &logos, &ethos, 1);
        assert!(result.is_consensus());
    }

    #[test]
    fn test_zero_confidence() {
        let engine = create_test_engine();

        let pathos = mock_response("Pathos", 0.0);
        let logos = mock_response("Logos", 0.0);
        let ethos = mock_response("Ethos", 0.0);

        let result = engine.evaluate(&pathos, &logos, &ethos, 1);
        assert!(!result.is_consensus());
        assert!(matches!(result, ConsensusResult::NeedsRevision { .. }));
    }

    #[test]
    fn test_extreme_threshold() {
        let config = ConsensusConfig {
            threshold: 0.99, // Very high threshold
            max_rounds: 1,
            ..Default::default()
        };
        let pathos = MockAgent {
            name: "Pathos",
            role: "Intent",
            model: "model1",
            confidence: 0.95,
        };
        let logos = MockAgent {
            name: "Logos",
            role: "Logic",
            model: "model2",
            confidence: 0.95,
        };
        let ethos = MockAgent {
            name: "Ethos",
            role: "Truth",
            model: "model3",
            confidence: 0.95,
        };
        let engine = ConsensusEngine::new(config, pathos, logos, ethos);

        let pathos = mock_response("Pathos", 0.95);
        let logos = mock_response("Logos", 0.95);
        let ethos = mock_response("Ethos", 0.95);

        let result = engine.evaluate(&pathos, &logos, &ethos, 1);
        assert!(!result.is_consensus());
    }

    #[test]
    fn test_custom_weights() {
        let config = ConsensusConfig {
            weights: AgentWeights {
                pathos: 0.50,
                logos: 0.30,
                ethos: 0.20,
            },
            ..Default::default()
        };
        let pathos = MockAgent {
            name: "Pathos",
            role: "Intent",
            model: "model1",
            confidence: 0.9,
        };
        let logos = MockAgent {
            name: "Logos",
            role: "Logic",
            model: "model2",
            confidence: 0.9,
        };
        let ethos = MockAgent {
            name: "Ethos",
            role: "Truth",
            model: "model3",
            confidence: 0.9,
        };
        let engine = ConsensusEngine::new(config, pathos, logos, ethos);

        // With custom weights: (1.0 * 0.5) + (0.0 * 0.3) + (0.0 * 0.2) = 0.5
        let aggregate = engine.calculate_aggregate(1.0, 0.0, 0.0);
        assert!((aggregate - 0.5).abs() < 0.001);
    }
}
