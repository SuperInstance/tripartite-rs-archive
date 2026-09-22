# Migration Guide: synesis-core → tripartite-rs

This guide helps you migrate from using `synesis-core`'s consensus engine to the standalone `tripartite-rs` crate.

## Quick Migration

The migration involves three main changes:
1. Update `Cargo.toml` dependency
2. Update imports
3. Define your own agent types (instead of using pre-defined ones)

## Step 1: Update Cargo.toml

### Before

```toml
[dependencies]
synesis-core = "0.2"
```

### After

```toml
[dependencies]
tripartite-rs = "0.1"
```

## Step 2: Update Imports

### Before

```rust
use synesis_core::consensus::{
    ConsensusEngine, ConsensusConfig, AgentWeights,
    ConsensusResult, ConsensusOutcome
};
use synesis_core::agents::{
    Agent, AgentInput, AgentOutput,
    PathosAgent, LogosAgent, EthosAgent,
};
use synesis_core::manifest::A2AManifest;
```

### After

```rust
use tripartite::{
    ConsensusEngine, ConsensusConfig, AgentWeights,
    ConsensusResult, ConsensusOutcome
};
use tripartite::{
    Agent, AgentInput, AgentOutput,
};
// You'll define PathosAgent, LogosAgent, EthosAgent yourself
```

## Step 3: Define Agent Types

### Before (synesis-core)

```rust
// synesis-core provides pre-defined agent types
use synesis_core::agents::{PathosAgent, LogosAgent, EthosAgent};

let pathos = PathosAgent::new(AgentConfig::default());
let logos = LogosAgent::new(AgentConfig::default());
let ethos = EthosAgent::new(AgentConfig::default());

let engine = ConsensusEngine::with_agents(pathos, logos, ethos);
```

### After (tripartite-rs)

```rust
use async_trait::async_trait;
use tripartite::{Agent, AgentInput, AgentOutput, Error};

// Define Pathos agent
#[derive(Clone)]
pub struct PathosAgent {
    model: String,
}

impl PathosAgent {
    pub fn new() -> Self {
        Self {
            model: "phi-3".to_string(),
        }
    }

    pub fn with_phi3() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for PathosAgent {
    fn name(&self) -> &str {
        "Pathos"
    }

    fn role(&self) -> &str {
        "Intent extraction and user understanding"
    }

    async fn process(&self, input: AgentInput) -> Result<AgentOutput, Error> {
        // Your Pathos logic here
        // Extract intent, disambiguate prompts, etc.

        Ok(AgentOutput {
            agent: self.name().to_string(),
            content: "Extracted intent".to_string(),
            confidence: 0.9,
            reasoning: Some("User wants X".to_string()),
            tokens_used: 100,
            latency_ms: 50,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model(&self) -> &str {
        &self.model
    }
}

// Define Logos agent
#[derive(Clone)]
pub struct LogosAgent {
    knowledge_base: String,
}

impl LogosAgent {
    pub fn new() -> Self {
        Self {
            knowledge_base: "default".to_string(),
        }
    }
}

#[async_trait]
impl Agent for LogosAgent {
    fn name(&self) -> &str {
        "Logos"
    }

    fn role(&self) -> &str {
        "Reasoning and solution synthesis"
    }

    async fn process(&self, input: AgentInput) -> Result<AgentOutput, Error> {
        // Your Logos logic here
        // RAG retrieval, solution synthesis, etc.

        Ok(AgentOutput {
            agent: self.name().to_string(),
            content: "Synthesized solution".to_string(),
            confidence: 0.92,
            reasoning: Some("Solution: X because Y".to_string()),
            tokens_used: 200,
            latency_ms: 100,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model(&self) -> &str {
        "logos-v1"
    }
}

// Define Ethos agent
#[derive(Clone)]
pub struct EthosAgent {
    strictness: f32,
}

impl EthosAgent {
    pub fn new() -> Self {
        Self {
            strictness: 0.85,
        }
    }
}

#[async_trait]
impl Agent for EthosAgent {
    fn name(&self) -> &str {
        "Ethos"
    }

    fn role(&self) -> &str {
        "Safety verification and fact-checking"
    }

    async fn process(&self, input: AgentInput) -> Result<AgentOutput, Error> {
        // Your Ethos logic here
        // Safety checks, fact verification, etc.

        Ok(AgentOutput {
            agent: self.name().to_string(),
            content: "Verified response".to_string(),
            confidence: 0.88,
            reasoning: Some("Response is safe and accurate".to_string()),
            tokens_used: 150,
            latency_ms: 75,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn model(&self) -> &str {
        "ethos-v1"
    }
}

// Create engine with your agents
let pathos = PathosAgent::new();
let logos = LogosAgent::new();
let ethos = EthosAgent::new();

let config = ConsensusConfig {
    threshold: 0.85,
    max_rounds: 3,
    weights: AgentWeights {
        pathos: 0.25,
        logos: 0.45,
        ethos: 0.30,
    },
};

let engine = ConsensusEngine::new(
    vec![
        Box::new(pathos),
        Box::new(logos),
        Box::new(ethos),
    ],
    config
);
```

## Step 4: Update AgentInput/AgentOutput

### Before (synesis-core)

```rust
use synesis_core::agents::{AgentInput, AgentOutput};
use synesis_core::manifest::A2AManifest;

let input = AgentInput {
    manifest: A2AManifest::new(prompt),
    context: HashMap::new(),
};

let output = AgentOutput {
    agent: "Pathos".to_string(),
    content: "Response".to_string(),
    confidence: 0.9,
    reasoning: None,
    tokens_used: 100,
    latency_ms: 50,
    metadata: HashMap::new(),
    vote: None,
};
```

### After (tripartite-rs)

```rust
use tripartite::{AgentInput, AgentOutput};

// AgentInput is now generic
// You can define your own Manifest type or use a simple String

let input = AgentInput {
    prompt: "User query".to_string(),
    context: std::collections::HashMap::new(),
};

let output = AgentOutput {
    agent: "Pathos".to_string(),
    content: "Response".to_string(),
    confidence: 0.9,
    reasoning: None,
    tokens_used: 100,
    latency_ms: 50,
    metadata: std::collections::HashMap::new(),
};
```

## Complete Example

### Before (synesis-core)

```rust
use synesis_core::consensus::{ConsensusEngine, ConsensusConfig};
use synesis_core::agents::{PathosAgent, LogosAgent, EthosAgent, AgentConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AgentConfig::default();

    let pathos = PathosAgent::new(config.clone());
    let logos = LogosAgent::new(config.clone());
    let ethos = EthosAgent::new(config);

    let engine = ConsensusEngine::with_agents(pathos, logos, ethos);

    let outcome = engine.run("What is the tripartite council?").await?;

    println!("{}", outcome.content);

    Ok(())
}
```

### After (tripartite-rs)

```rust
use tripartite::{ConsensusEngine, ConsensusConfig, Agent, AgentInput, AgentOutput};
use async_trait::async_trait;

// Define your agents (see Step 3 for full implementations)
#[derive(Clone)]
pub struct PathosAgent;
#[derive(Clone)]
pub struct LogosAgent;
#[derive(Clone)]
pub struct EthosAgent;

impl Agent for PathosAgent { /* ... */ }
impl Agent for LogosAgent { /* ... */ }
impl Agent for EthosAgent { /* ... */ }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConsensusConfig::default();

    let pathos = PathosAgent;
    let logos = LogosAgent;
    let ethos = EthosAgent;

    let engine = ConsensusEngine::new(
        vec![
            Box::new(pathos),
            Box::new(logos),
            Box::new(ethos),
        ],
        config
    );

    let outcome = engine.run("What is the tripartite council?").await?;

    println!("{}", outcome.content);

    Ok(())
}
```

## Breaking Changes

### 1. Pre-defined Agent Types Removed

**Before**: synesis-core provides `PathosAgent`, `LogosAgent`, `EthosAgent`
**After**: You define your own agent types

**Rationale**: Makes tripartite-rs generic and reusable for any use case.

**Migration**: Copy agent implementations from synesis-core or define your own.

### 2. Manifest Type Generic

**Before**: `AgentInput` uses `A2AManifest` from synesis-core
**After**: `AgentInput` uses generic prompt/context structure

**Rationale**: Removes dependency on synesis-core types.

**Migration**: Use simple string prompts or define your own manifest type.

### 3. Agent Config Simplified

**Before**: Agents use `AgentConfig` from synesis-core
**After**: Agents are configured directly via struct fields

**Rationale**: Simpler, more flexible agent configuration.

**Migration**: Pass configuration directly to agent constructors.

## Minimal Working Example

Here's a minimal example to get you started:

```rust
use tripartite::{ConsensusEngine, ConsensusConfig, Agent, AgentInput, AgentOutput};
use async_trait::async_trait;

// Simple mock agents
struct MockAgent(&'static str, f32);

#[async_trait]
impl Agent for MockAgent {
    fn name(&self) -> &str { self.0 }
    fn role(&self) -> &str { "Mock agent" }

    async fn process(&self, _input: AgentInput) -> Result<AgentOutput, tripartite::Error> {
        Ok(AgentOutput {
            agent: self.name().to_string(),
            content: "Mock response".to_string(),
            confidence: self.1,
            reasoning: None,
            tokens_used: 10,
            latency_ms: 1,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn is_ready(&self) -> bool { true }
    fn model(&self) -> &str { "mock" }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConsensusConfig::default();

    let engine = ConsensusEngine::new(
        vec![
            Box::new(MockAgent("Agent1", 0.9)),
            Box::new(MockAgent("Agent2", 0.92)),
            Box::new(MockAgent("Agent3", 0.88)),
        ],
        config
    );

    let outcome = engine.run("Test prompt").await?;

    match outcome.result {
        tripartite::ConsensusResult::Reached { .. } => {
            println!("Consensus: {}", outcome.content);
        }
        _ => {
            println!("No consensus reached");
        }
    }

    Ok(())
}
```

## Testing Your Migration

After updating imports and defining agents, run your tests:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_consensus

# Expected: All tests pass
```

## Troubleshooting

### Issue: "Agent trait not satisfied"

**Error**:
```
the trait `Agent` is not implemented for `MyAgent`
```

**Solution**: Ensure your agent implements all required methods:
```rust
#[async_trait]
impl Agent for MyAgent {
    fn name(&self) -> &str { ... }
    fn role(&self) -> &str { ... }
    async fn process(&self, input: AgentInput) -> Result<AgentOutput, Error> { ... }
    fn is_ready(&self) -> bool { ... }
    fn model(&self) -> &str { ... }
}
```

### Issue: "Cannot find AgentInput"

**Error**:
```
cannot find type `AgentInput` in this scope
```

**Solution**: Import from tripartite:
```rust
use tripartite::{AgentInput, AgentOutput};
```

### Issue: "Wrong number of type arguments"

**Error**:
```
wrong number of type arguments: expected 1, found 0
```

**Solution**: AgentInput is not generic in tripartite-rs:
```rust
// Before (synesis-core)
let input: AgentInput<MyType> = ...;

// After (tripartite-rs)
let input: AgentInput = ...;
```

### Issue: "ConsensusEngine::with_agents not found"

**Error**:
```
no function or associated item named `with_agents` found
```

**Solution**: Use `ConsensusEngine::new` with a vector of agents:
```rust
// Before
let engine = ConsensusEngine::with_agents(pathos, logos, ethos);

// After
let engine = ConsensusEngine::new(
    vec![Box::new(pathos), Box::new(logos), Box::new(ethos)],
    config
);
```

## Workspace Migration

If you're migrating a workspace with multiple packages:

### Update workspace Cargo.toml

```toml
# In workspace root Cargo.toml
[workspace.dependencies]
# Old
# synesis-core = "0.2"

# New
tripartite-rs = "0.1"

# In each package's Cargo.toml
[dependencies]
tripartite-rs = { workspace = true }
```

### Update all imports across workspace

```bash
# Find all files using synesis-core
grep -r "use synesis_core" .

# Update imports in each file
# Use your editor's find-and-replace
```

### Run workspace tests

```bash
# Test entire workspace
cargo test --workspace

# Expected: All tests pass
```

## Summary

| Aspect | synesis-core | tripartite-rs |
|--------|--------------|---------------|
| **Agent Types** | Pre-defined (PathosAgent, etc.) | You define them |
| **Generic** | Specific to SuperInstance | Generic, reusable |
| **Dependencies** | Part of synesis-core workspace | Standalone crate |
| **API** | Specific to AI agents | Generic consensus |
| **Use Cases** | SuperInstance only | Any multi-agent system |

### Migration Checklist

- [ ] Update `Cargo.toml` dependency
- [ ] Update imports in all files
- [ ] Define custom agent types
- [ ] Update `AgentInput` usage
- [ ] Update `ConsensusEngine::new` calls
- [ ] Run all tests
- [ ] Verify functionality

### Benefits of Migration

1. **Generic & Reusable**: Use consensus in any project
2. **Lightweight**: No SuperInstance-specific dependencies
3. **Flexible**: Define agents for any domain
4. **Standalone**: Can be published to crates.io independently
5. **Maintainable**: Clear separation of concerns

## Need Help?

If you encounter issues during migration:

1. **Check imports**: Ensure all `synesis_core` references are updated
2. **Check version**: Verify you're using `tripartite-rs = "0.1"` or later
3. **Review examples**: See `/examples` directory in tripartite-rs repo
4. **Read docs**: Check https://docs.rs/tripartite-rs
5. **Report issues**: https://github.com/SuperInstance/tripartite-rs/issues

## Advanced Migration

### Gradual Migration (Both Crates)

During migration, you can use both crates:

```rust
// Use aliases to distinguish
use synesis_core as old;
use tripartite as new;

// This is temporary - remove old code as you migrate
```

**Warning**: Don't run both in production long-term. They have separate agent instances and won't share state.

### Feature Migration

If you were using synesis-core features beyond consensus:

```rust
// synesis-core provides:
// - Consensus engine → Use tripartite-rs
// - Agent implementations → Define your own
// - Manifest types → Define your own
// - Privacy integration → Use privox crate
// - Knowledge vault → Use synesis-knowledge (or extract later)
```

Each feature will be extracted to its own crate:
- **Consensus**: tripartite-rs (this crate)
- **Privacy**: privox (already extracted)
- **Knowledge**: TBD (future extraction)
- **Models**: TBD (future extraction)

---

**Version**: 0.1.0
**Last Updated**: 2026-01-08
**For**: synesis-core 0.2.x users
