//! ML model ensemble example
//!
//! Demonstrates using tripartite-rs for ML ensemble learning:
//! - Multiple ML models (Random Forest, SVM, Neural Network)
//! - Weighted voting for classification
//! - Single round (typical for ML ensembles)

use async_trait::async_trait;
use std::sync::Arc;
use tripartite::{Agent, AgentInput, AgentOutput, AgentWeights, ConsensusConfig, ConsensusEngine};

/// ML model that makes predictions
struct MLModel {
    name: String,
    prediction: String,
    confidence: f32,
}

#[async_trait]
impl Agent for MLModel {
    async fn process(&self, _input: AgentInput) -> Result<AgentOutput, tripartite::Error> {
        Ok(
            AgentOutput::new(&self.name, self.prediction.clone(), self.confidence)
                .with_reasoning(format!("Based on training data and {} features", 1000)),
        )
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn role(&self) -> &str {
        "ML classifier"
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model(&self) -> &str {
        "ml-model"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("=== ML Ensemble Example ===\n");

    // Create three ML models with different predictions
    let rf_model = Arc::new(MLModel {
        name: "Random Forest".to_string(),
        prediction: "spam".to_string(),
        confidence: 0.92,
    });

    let svm_model = Arc::new(MLModel {
        name: "SVM".to_string(),
        prediction: "spam".to_string(),
        confidence: 0.88,
    });

    let nn_model = Arc::new(MLModel {
        name: "Neural Network".to_string(),
        prediction: "ham".to_string(), // Disagrees!
        confidence: 0.65,
    });

    println!("Created 3 ML models:");
    println!(
        "  - {}: predicts '{}' with {:.2} confidence",
        rf_model.name, rf_model.prediction, rf_model.confidence
    );
    println!(
        "  - {}: predicts '{}' with {:.2} confidence",
        svm_model.name, svm_model.prediction, svm_model.confidence
    );
    println!(
        "  - {}: predicts '{}' with {:.2} confidence",
        nn_model.name, nn_model.prediction, nn_model.confidence
    );
    println!();

    // Configure consensus for ML ensemble
    // Single round is typical for ML (no revision)
    let config = ConsensusConfig {
        threshold: 0.85,                  // 85% confidence required
        max_rounds: 1,                    // Single round (no revision for ML)
        weights: AgentWeights::default(), // Default weights
    };

    println!("Ensemble Configuration:");
    println!("  - Threshold: {:.0}", config.threshold * 100.0);
    println!(
        "  - Max Rounds: {} (ML ensembles typically use single round)",
        config.max_rounds
    );
    println!("  - Weights: Default");
    println!();

    let mut engine = ConsensusEngine::new(config, rf_model, svm_model, nn_model);

    println!("Running ensemble on input: \"Free money!!!\"\n");

    let outcome = engine.run("Classify: Free money!!!").await?;

    println!("=== Results ===");
    println!("Final Prediction: {}", outcome.content);
    println!("Consensus Reached: {}", outcome.is_consensus());
    println!(
        "Aggregate Confidence: {:.2}",
        outcome.aggregate_confidence().unwrap_or(0.0)
    );
    println!("Duration: {}ms", outcome.total_duration_ms);

    // Calculate aggregate manually
    // (0.92 + 0.88 + 0.65) / 3 = 0.817
    let expected_aggregate = (0.92 + 0.88 + 0.65) / 3.0;
    println!("Expected Aggregate: {:.3}", expected_aggregate);

    println!("\n=== Analysis ===");

    if outcome.is_consensus() {
        println!("✓ Ensemble reached consensus on '{}'", outcome.content);
        println!("  2 out of 3 models agreed with high confidence");
    } else {
        println!("✗ Ensemble did not reach consensus");
        println!("  Models disagreed or confidence was too low");
        if let Some(ref reasoning) = outcome.reasoning {
            println!("  Reason: {}", reasoning);
        }
    }

    println!("\n=== Why Ensemble Matters ===");
    println!("1. Random Forest: Great at handling noisy data");
    println!("2. SVM: Good at finding decision boundaries");
    println!("3. Neural Network: Excels at pattern recognition");
    println!();
    println!("By combining all three, we get:");
    println!("  - Better generalization");
    println!("  - Reduced overfitting");
    println!("  - More robust predictions");

    println!("\n=== Real-World Use Cases ===");
    println!("- Spam detection");
    println!("- Fraud detection");
    println!("- Medical diagnosis");
    println!("- Image classification");
    println!("- Sentiment analysis");

    Ok(())
}
