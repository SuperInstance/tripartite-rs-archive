// Benchmark consensus performance with varying configurations
//
// These benchmarks measure:
// - Consensus speed with 3 agents (standard tripartite)
// - Multi-round consensus performance
// - Different threshold configurations

use async_trait::async_trait;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;
use tripartite::{Agent, AgentInput, AgentOutput, ConsensusConfig, ConsensusEngine};

// Mock agent for benchmarking
struct MockAgent {
    name: String,
    latency_ms: u64,
}

#[async_trait]
impl Agent for MockAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn role(&self) -> &str {
        "Mock agent for benchmarking"
    }

    async fn process(&self, _input: AgentInput) -> tripartite::Result<AgentOutput> {
        // Simulate processing latency
        if self.latency_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.latency_ms)).await;
        }

        Ok(AgentOutput::new(
            &self.name,
            format!("Response from {}", self.name),
            0.9,
        ))
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model(&self) -> &str {
        "mock-model"
    }
}

fn create_mock_agent(name: String, latency_ms: u64) -> std::sync::Arc<MockAgent> {
    std::sync::Arc::new(MockAgent { name, latency_ms })
}

fn bench_consensus_three_agents(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus/3_agents");

    // Benchmark with zero latency
    group.bench_function("zero_latency", |b| {
        let agent_1 = create_mock_agent("agent_0".to_string(), 0);
        let agent_2 = create_mock_agent("agent_1".to_string(), 0);
        let agent_3 = create_mock_agent("agent_2".to_string(), 0);

        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let mut engine =
                    ConsensusEngine::with_agents(agent_1.clone(), agent_2.clone(), agent_3.clone());

                let _outcome = engine.run("Test query").await;
            });
    });

    // Benchmark with 1ms latency per agent
    group.bench_function("1ms_latency", |b| {
        let agent_1 = create_mock_agent("agent_0".to_string(), 1);
        let agent_2 = create_mock_agent("agent_1".to_string(), 1);
        let agent_3 = create_mock_agent("agent_2".to_string(), 1);

        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let mut engine =
                    ConsensusEngine::with_agents(agent_1.clone(), agent_2.clone(), agent_3.clone());

                let _outcome = engine.run("Test query").await;
            });
    });

    group.finish();
}

fn bench_consensus_multi_round(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus/multi_round");

    // Benchmark with different thresholds (affects number of rounds)
    for threshold in [0.5, 0.7, 0.85, 0.95].iter() {
        group.bench_with_input(
            BenchmarkId::new("threshold", threshold),
            threshold,
            |b, &threshold| {
                let agent_1 = create_mock_agent("agent_0".to_string(), 0);
                let agent_2 = create_mock_agent("agent_1".to_string(), 0);
                let agent_3 = create_mock_agent("agent_2".to_string(), 0);

                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async {
                        let config = ConsensusConfig {
                            threshold,
                            max_rounds: 5,
                            weights: tripartite::AgentWeights::default(),
                        };
                        let mut engine = ConsensusEngine::new(
                            config,
                            agent_1.clone(),
                            agent_2.clone(),
                            agent_3.clone(),
                        );

                        let _outcome = engine.run("Test query").await;
                    });
            },
        );
    }

    group.finish();
}

fn bench_consensus_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus/latency");

    for latency_ms in [0, 1, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(latency_ms),
            latency_ms,
            |b, &latency_ms| {
                let agent_1 = create_mock_agent("agent_0".to_string(), latency_ms);
                let agent_2 = create_mock_agent("agent_1".to_string(), latency_ms);
                let agent_3 = create_mock_agent("agent_2".to_string(), latency_ms);

                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async {
                        let mut engine = ConsensusEngine::with_agents(
                            agent_1.clone(),
                            agent_2.clone(),
                            agent_3.clone(),
                        );

                        let _outcome = engine.run("Test query").await;
                    });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_consensus_three_agents,
    bench_consensus_multi_round,
    bench_consensus_latency
);
criterion_main!(benches);
