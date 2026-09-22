# SuperInstance Integration Plan: tripartite-rs

This document outlines how SuperInstance will integrate the standalone `tripartite-rs` crate to replace the consensus engine currently in `synesis-core`.

## Overview

**Current State**: SuperInstance uses consensus engine embedded in `synesis-core`
**Target State**: SuperInstance uses `tripartite-rs` as a dependency
**Status**: Awaiting tripartite-rs extraction completion (Agents 2.1, 2.2, 2.3)
**Agent**: 2.4 (Publishing & Integration Specialist)

## Integration Strategy

### Phase 1: Development (Path Dependency)

During development, SuperInstance will use a local path dependency:

```toml
# In /mnt/c/claudesuperinstance/Cargo.toml
[workspace.dependencies]
# Local development
tripartite-rs = { path = "../tripartite-rs" }

# In crates/synesis-core/Cargo.toml
[dependencies]
tripartite-rs = { workspace = true }
```

**Benefits**:
- Instant feedback when making changes
- No need to publish to test
- Can modify both repos simultaneously

**Duration**: Until tripartite-rs is published to crates.io

### Phase 2: Production (crates.io Dependency)

After tripartite-rs is published:

```toml
# In /mnt/c/claudesuperinstance/Cargo.toml
[workspace.dependencies]
# Published version
tripartite-rs = "0.1"

# In crates/synesis-core/Cargo.toml
[dependencies]
tripartite-rs = { workspace = true }
```

**Benefits**:
- Stable version
- Semver guarantees
- Works for external users

## Files Requiring Updates

### 1. Workspace Configuration

**File**: `/mnt/c/claudesuperinstance/Cargo.toml`

**Changes**:
```toml
[workspace]
members = [
    "crates/synesis-cli",
    "crates/synesis-core",
    "crates/synesis-knowledge",
    "crates/synesis-models",
    "crates/synesis-privacy",
    "crates/synesis-cloud",
]

[workspace.dependencies]
# Add tripartite-rs dependency
tripartite-rs = { path = "../tripartite-rs" }  # Phase 1
# tripartite-rs = "0.1"  # Phase 2

# Keep existing dependencies
synesis-knowledge = { path = "crates/synesis-knowledge" }
synesis-models = { path = "crates/synesis-models" }
synesis-privacy = { path = "crates/synesis-privacy" }
synesis-cloud = { path = "crates/synesis-cloud" }
tokio = { version = "1", features = ["full"] }
# ... etc
```

### 2. synesis-core Dependencies

**File**: `/mnt/c/claudesuperinstance/crates/synesis-core/Cargo.toml`

**Changes**:
```toml
[dependencies]
# Add tripartite-rs
tripartite-rs = { workspace = true }

# Keep other dependencies (can remove consensus-specific ones)
tokio = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }
# ... etc

# Remove if no longer needed:
# (these might be used by other modules, so check carefully)
# async-trait = "0.1"  # Only if not used elsewhere
```

### 3. synesis-core Module Structure

**File**: `/mnt/c/claudesuperinstance/crates/synesis-core/src/lib.rs`

**Before**:
```rust
pub mod agents;
pub mod consensus;
pub mod council;
pub mod error;
pub mod manifest;
pub mod metrics;
```

**After**:
```rust
pub mod agents;
// pub mod consensus;  // REMOVE: Now using tripartite-rs
pub mod council;
pub mod error;
pub mod manifest;
pub mod metrics;

// Re-export tripartite-rs for convenience
pub use tripartite::{
    ConsensusEngine, ConsensusConfig, ConsensusResult, ConsensusOutcome,
    Agent, AgentInput, AgentOutput, AgentWeights,
};
```

### 4. synesis-core Agents Module

**File**: `/mnt/c/claudesuperinstance/crates/synesis-core/src/agents/mod.rs`

**Changes**:
```rust
// Before: Use local consensus
use crate::consensus::{ConsensusEngine, ConsensusConfig};

// After: Use tripartite-rs
use tripartite::{ConsensusEngine, ConsensusConfig, Agent, AgentInput, AgentOutput};

// Keep PathosAgent, LogosAgent, EthosAgent
// They now implement the tripartite::Agent trait

pub struct PathosAgent { ... }
pub struct LogosAgent { ... }
pub struct EthosAgent { ... }

// Implement tripartite::Agent instead of local Agent trait
#[async_trait]
impl Agent for PathosAgent {
    fn name(&self) -> &str { "Pathos" }
    fn role(&self) -> &str { "Intent extraction" }

    async fn process(&self, input: AgentInput) -> Result<AgentOutput, tripartite::Error> {
        // ... existing logic
    }

    fn is_ready(&self) -> bool { true }
    fn model(&self) -> &str { "phi-3" }
}

// Similar for LogosAgent and EthosAgent
```

### 5. synesis-core Council Module

**File**: `/mnt/c/claudesuperinstance/crates/synesis-core/src/council.rs`

**Changes**:
```rust
// Before
use crate::consensus::ConsensusEngine;

// After
use tripartite::ConsensusEngine;

pub struct TripartiteCouncil {
    consensus: ConsensusEngine,
    // ... other fields
}

impl TripartiteCouncil {
    pub fn new(/* ... */) -> Result<Self, Error> {
        // Create agents
        let pathos = PathosAgent::new(...);
        let logos = LogosAgent::new(...);
        let ethos = EthosAgent::new(...);

        // Create consensus engine
        let config = ConsensusConfig::default();
        let consensus = ConsensusEngine::new(
            vec![
                Box::new(pathos),
                Box::new(logos),
                Box::new(ethos),
            ],
            config
        );

        Ok(Self { consensus, ... })
    }

    pub async fn process(&mut self, prompt: &str) -> Result<ConsensusOutcome, Error> {
        self.consensus.run(prompt).await
    }
}
```

### 6. CLI Commands

**File**: `/mnt/c/claudesuperinstance/crates/synesis-cli/src/commands/*.rs`

**Changes**: Update imports in all command files using consensus:

```rust
// Before
use synesis_core::consensus::{ConsensusEngine, ConsensusConfig};

// After
use synesis_core::{ConsensusEngine, ConsensusConfig};  // Re-exported
// OR
use tripartite::{ConsensusEngine, ConsensusConfig};
```

**Affected files**:
- `src/commands/model.rs`
- `src/commands/cloud.rs`
- Any other command using consensus directly

## Removal Plan

After integration is complete and tests pass, remove old consensus code:

### Files to Delete

```
/mnt/c/claudesuperinstance/crates/synesis-core/src/consensus/
├── mod.rs  # DELETE: All consensus logic moved to tripartite-rs
```

### Code to Remove

**File**: `/mnt/c/claudesuperinstance/crates/synesis-core/src/lib.rs`

```rust
// REMOVE this line:
pub mod consensus;
```

**File**: `/mnt/c/claudesuperinstance/crates/synesis-core/Cargo.toml`

```rust
# Check if these dependencies are only used by consensus:
# If yes, remove them; if no, keep them
# async-trait = "0.1"  # REMOVE if only consensus uses it
```

## Testing Strategy

### 1. Unit Tests

```bash
# Test tripartite-rs in isolation
cd /mnt/c/claudesuperinstance/tripartite-rs
cargo test --all-features

# Expected: 30+ tests pass
```

### 2. Integration Tests

```bash
# Test synesis-core with tripartite-rs
cd /mnt/c/claudesuperinstance
cargo test --package synesis-core

# Expected: 85+ tests pass
# (Some tests may need updating)
```

### 3. Workspace Tests

```bash
# Test entire workspace
cargo test --workspace

# Expected: 250+ tests pass
# (Current baseline: 250+ tests)
```

### 4. CLI Tests

```bash
# Test CLI functionality
cargo run -- synesis status
cargo run -- synesis ask "What is the tripartite council?"
cargo run -- synesis model list

# Expected: All commands work correctly
```

## Migration Steps

### Step 1: Add Dependency (Day 1)

```bash
# Edit workspace Cargo.toml
cd /mnt/c/claudesuperinstance
vim Cargo.toml

# Add tripartite-rs to workspace.dependencies
tripartite-rs = { path = "../tripartite-rs" }
```

### Step 2: Update Imports (Day 1-2)

```bash
# Find all files using consensus
grep -r "use synesis_core::consensus" crates/

# Update each file:
# - Change imports to tripartite
# - Update Agent trait implementations
# - Test each module
```

### Step 3: Re-exports (Day 2)

```bash
# Add re-exports to synesis-core/src/lib.rs
# This minimizes changes to dependent crates

vim crates/synesis-core/src/lib.rs

# Add:
pub use tripartite::{
    ConsensusEngine, ConsensusConfig, ConsensusResult, ConsensusOutcome,
    Agent, AgentInput, AgentOutput, AgentWeights,
};
```

### Step 4: Update Agent Implementations (Day 2-3)

```bash
# Update agent trait implementations
vim crates/synesis-core/src/agents/mod.rs
vim crates/synesis-core/src/agents/pathos.rs
vim crates/synesis-core/src/agents/logos.rs
vim crates/synesis-core/src/agents/ethos.rs

# Change:
# - impl crate::agents::Agent → impl tripartite::Agent
# - Update method signatures if needed
# - Update return types
```

### Step 5: Test Incrementally (Day 3-4)

```bash
# Test each module
cargo test --package synesis-core --lib agents
cargo test --package synesis-core --lib council
cargo test --package synesis-core --lib consensus

# Test workspace
cargo test --workspace
```

### Step 6: Remove Old Code (Day 5)

```bash
# Only after all tests pass
rm -rf crates/synesis-core/src/consensus/

# Update lib.rs to remove consensus module
vim crates/synesis-core/src/lib.rs

# Final test
cargo test --workspace
```

### Step 7: Documentation Updates (Day 5)

```bash
# Update CLAUDE.md
vim CLAUDE.md

# Update architecture docs
vim ARCHITECTURE.md

# Update dependency diagram
# Add tripartite-rs to ecosystem docs
```

## Potential Breaking Changes

### None Expected!

The integration is designed to be **non-breaking**:

1. **Re-exports**: synesis-core re-exports tripartite types
2. **Same API**: Agent trait has same methods
3. **Same behavior**: Consensus logic is identical

### Minor API Adjustments

These may be needed but shouldn't break existing code:

1. **Agent trait method names**: Ensure alignment
2. **Error types**: Use tripartite::Error or wrap it
3. **AgentInput structure**: May need small adjustments

## Rollback Plan

If integration causes issues:

```bash
# 1. Revert changes
cd /mnt/c/claudesuperinstance
git checkout main
git reset --hard HEAD~1

# 2. Restore consensus module
git checkout HEAD~1 -- crates/synesis-core/src/consensus/

# 3. Remove tripartite-rs dependency
vim Cargo.toml  # Remove tripartite-rs line

# 4. Revert imports
git checkout HEAD~1 -- crates/synesis-core/src/

# 5. Test
cargo test --workspace
```

## Success Criteria

Integration is successful when:

- ✅ All 250+ tests pass
- ✅ Zero compiler warnings
- ✅ CLI commands work correctly
- ✅ No performance regression
- ✅ Documentation updated
- ✅ Old consensus code removed
- ✅ Git history clean

## Timeline

| Day | Task | Status |
|-----|------|--------|
| 1 | Add tripartite-rs dependency | Pending (awaiting extraction) |
| 1-2 | Update imports across codebase | Pending |
| 2 | Add re-exports to synesis-core | Pending |
| 2-3 | Update agent implementations | Pending |
| 3-4 | Test and fix issues | Pending |
| 5 | Remove old consensus code | Pending |
| 5 | Update documentation | Pending |

**Total Duration**: 5 days (after tripartite-rs extraction is complete)

## Dependencies

This integration depends on:

1. **Agent 2.1**: Create tripartite-rs crate structure
2. **Agent 2.2**: Extract consensus logic
3. **Agent 2.3**: Implement generic agent trait
4. **Agent 2.4**: Create publishing documentation (this file)

**Blockers**: None (can proceed in parallel with Agents 2.1-2.3)

## Notes

- **Path dependency first**: Use local path during development
- **crates.io later**: Switch to published version after v0.1.0 release
- **Test frequently**: Run tests after each step
- **Keep backups**: Commit after each successful step
- **Document changes**: Update CLAUDE.md as you go

---

**Created**: 2026-01-08
**Agent**: 2.4 (Publishing & Integration Specialist)
**Status**: Ready for integration phase
**Next Steps**: Await tripartite-rs extraction completion (Agents 2.1-2.3)
