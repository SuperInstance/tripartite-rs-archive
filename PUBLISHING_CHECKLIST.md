# Publishing Checklist for tripartite-rs

This checklist guides you through publishing `tripartite-rs` to crates.io as a standalone crate.

## Pre-Publish Requirements

Before publishing, ensure all of the following are complete:

- [ ] **Source Code Extracted**: All consensus engine code copied from synesis-core
- [ ] **All Tests Pass**: Run `cargo test --all-features` (100% pass rate required)
- [ ] **Zero Compiler Warnings**: Run `cargo clippy --all-features -- -D warnings`
- [ ] **Code Formatted**: Run `cargo fmt --all`
- [ ] **README Complete**: Converts visitors in 10 seconds, shows quick start
- [ ] **All Examples Run**: `cargo run --example basic` (if examples exist)
- [ ] **CI/CD Configured**: GitHub Actions workflows pass on all platforms
- [ ] **Documentation Complete**: All public APIs have doc comments
- [ ] **Cross-References Added**: Links to SuperInstance use cases
- [ ] **LICENSE File Present**: MIT OR Apache-2.0
- [ ] **CONTRIBUTING.md Present**: Contribution guidelines
- [ ] **Cargo.toml Metadata**: Complete metadata (description, categories, keywords)

## Pre-Publish Verification

Run these commands to verify the crate is ready:

```bash
# Navigate to tripartite-rs directory
cd /mnt/c/claudesuperinstance/tripartite-rs

# 1. Run all tests with all features
cargo test --all-features

# Expected: All tests pass (30+ tests)
# synesis-core has 85 tests in consensus module
# tripartite-rs should have similar coverage

# 2. Check for compiler warnings
cargo clippy --all-features -- -D warnings

# Expected: Zero warnings
# Any warning must be fixed before publishing

# 3. Format check
cargo fmt -- --check

# Expected: No formatting errors
# If errors, run: cargo fmt

# 4. Build documentation
cargo doc --all-features --no-deps

# Expected: Documentation builds successfully
# Check for missing docs: cargo doc --all-features --no-deps 2>&1 | grep "missing"

# 5. Run examples (if any)
cargo run --example basic
cargo run --example three_agents
cargo run --example custom_weights

# Expected: All examples run without errors

# 6. Verify package metadata
cargo package --list

# Expected: All necessary files included
# - LICENSE, LICENSE-MIT, LICENSE-APACHE
# - README.md
# - Cargo.toml
# - src/**/*.rs
```

## Cargo.toml Verification

Ensure your `Cargo.toml` has complete metadata:

```toml
[package]
name = "tripartite-rs"
version = "0.1.0"
description = "Generic multi-agent consensus system for Rust"
license = "MIT OR Apache-2.0"
repository = "https://github.com/SuperInstance/tripartite-rs"
authors = ["SuperInstance Team"]
edition = "2021"
readme = "README.md"
keywords = ["consensus", "multi-agent", "voting", "ai", "async"]
categories = [
    "asynchronous",
    "concurrency",
    "science",
    "algorithms",
]
rust-version = "1.75"  # Minimum supported Rust version

[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
thiserror = "1"
anyhow = "1"

# Async trait
async-trait = "0.1"

# Logging
tracing = "0.1"

# Time
chrono = { version = "0.4", features = ["serde"] }

# IDs
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tokio-test = "0.4"

[lib]
name = "tripartite"
path = "src/lib.rs"

[features]
default = []
```

## Test on crates.io (Dry Run)

Before publishing for real, do a dry run:

```bash
# Navigate to tripartite-rs
cd /mnt/c/claudesuperinstance/tripartite-rs

# Dry run (validates package without publishing)
cargo publish --dry-run

# Expected output:
# - Packaging tripartite-rs v0.1.0
# - Verifying tripartite-rs v0.1.0
# - (success message)

# If dry run fails:
# 1. Read error messages carefully
# 2. Fix issues (missing files, warnings, etc.)
# 3. Run cargo test again
# 4. Run cargo publish --dry-run again
```

## Common Dry Run Issues

### Issue: "can't find crate"
```
error: cannot find crate `synesis-core` in the registry
```
**Solution**: Remove any `synesis-core` dependencies from `Cargo.toml`. The crate must be standalone.

### Issue: Missing documentation
```
warning: unused doc comment
```
**Solution**: Add doc comments to all public types and functions:
```rust
/// This is the main consensus engine
pub struct ConsensusEngine { ... }
```

### Issue: Warning about dead code
```
warning: field is never read
```
**Solution**: Either use the field or mark it as `pub` if it's part of the public API.

### Issue: Missing license files
```
error: failed to verify package tarball
```
**Solution**: Ensure `LICENSE-MIT` and `LICENSE-APACHE` files exist in the repository root.

## Publish for Real

Once dry run succeeds, publish:

```bash
# Publish to crates.io
cargo publish

# Expected output:
# - Uploading tripartite-rs v0.1.0
# - Published tripartite-rs v0.1.0
# - crates.io URL: https://crates.io/crates/tripartite-rs

# NOTE: First-time publish may require:
# 1. Login to crates.io (creates API token)
# 2. Running: cargo login
# 3. Pasting API token
```

## Post-Publish Checklist

After publishing, verify everything:

- [ ] **Verify on crates.io**
  - Visit: https://crates.io/crates/tripartite-rs
  - Check metadata (description, keywords, categories)
  - Verify license is correct
  - Check version is 0.1.0

- [ ] **Verify docs.rs builds**
  - Visit: https://docs.rs/tripartite-rs
  - Wait 5-10 minutes for docs to build (automatic)
  - Check all public types are documented
  - Verify examples render correctly

- [ ] **Create GitHub Release v0.1.0**
  - Go to: https://github.com/SuperInstance/tripartite-rs/releases
  - Click "Draft a new release"
  - Tag: `v0.1.0`
  - Title: `tripartite-rs v0.1.0`
  - Description: Use content from `RELEASE_NOTES.md`
  - Attach binaries (if any)

- [ ] **Update SuperInstance to use published version**
  - Edit `/mnt/c/claudesuperinstance/Cargo.toml`
  - Replace `synesis-core` consensus with `tripartite-rs` dependency
  - See `MIGRATION_GUIDE.md` for details

- [ ] **Update ecosystem documentation**
  - Update `ECOSYSTEM_INDEX.md`
  - Update `CLAUDE.md` with tripartite-rs reference
  - Update `ARCHITECTURE.md` with new dependency diagram

## Migration from synesis-core

Users migrating from `synesis-core` consensus to `tripartite-rs` need these changes:

### 1. Update Cargo.toml

```toml
# Old
[dependencies]
synesis-core = "0.2"

# New
[dependencies]
tripartite-rs = "0.1"
```

### 2. Update Imports

```rust
// Old
use synesis_core::consensus::{ConsensusEngine, ConsensusConfig, AgentWeights};
use synesis_core::agents::{Agent, AgentInput, AgentOutput};

// New
use tripartite::{ConsensusEngine, ConsensusConfig, AgentWeights};
use tripartite::{Agent, AgentInput, AgentOutput};
```

### 3. Update Code (Minimal Changes)

The API is 99% compatible. The main difference:
- `synesis-core` has specific agent types (PathosAgent, LogosAgent, EthosAgent)
- `tripartite-rs` is generic - you define your own agent types

See `MIGRATION_GUIDE.md` for complete migration instructions.

## Verification Steps

After publishing, verify the following:

### 1. Install from crates.io

```bash
# Create a new test project
cargo new test-tripartite
cd test-tripartite

# Add dependency
echo 'tripartite-rs = "0.1"' >> Cargo.toml

# Build
cargo build

# Expected: Downloads and compiles tripartite-rs v0.1.0 successfully
```

### 2. Test Basic Functionality

```rust
// In src/main.rs
use tripartite::{ConsensusEngine, ConsensusConfig, Agent, AgentInput, AgentOutput};
use async_trait::async_trait;

struct TestAgent;

#[async_trait]
impl Agent for TestAgent {
    fn name(&self) -> &str { "TestAgent" }
    fn role(&self) -> &str { "Test" }

    async fn process(&self, input: AgentInput) -> Result<AgentOutput, tripartite::Error> {
        Ok(AgentOutput::new("TestAgent", "Response".to_string(), 0.9))
    }

    fn is_ready(&self) -> bool { true }
    fn model(&self) -> &str { "test" }
}

fn main() {
    println!("tripartite-rs imported successfully!");
}
```

```bash
cargo run

# Expected: Prints "tripartite-rs imported successfully!"
```

### 3. Documentation Check

```bash
# Generate docs
cargo doc --open

# Expected: Opens browser with documentation
# Check all public types are documented
```

### 4. SuperInstance Integration

```bash
# Navigate to SuperInstance
cd /mnt/c/claudesuperinstance

# Update Cargo.toml to use tripartite-rs
# (See MIGRATION_GUIDE.md)

# Build workspace
cargo build --workspace

# Expected: Builds successfully with tripartite-rs dependency

# Run tests
cargo test --workspace

# Expected: All 250+ tests pass
```

## Publishing Timeline

### Phase 1: Preparation (Agent 2.1, 2.2, 2.3)
- ✅ Extract consensus code from synesis-core
- ✅ Create tripartite-rs crate structure
- ✅ Implement generic agent trait
- ✅ Port all consensus logic
- ✅ Add tests (30+ tests)
- ✅ Create documentation

### Phase 2: Pre-Publish Verification (Agent 2.4)
- ✅ Create PUBLISHING_CHECKLIST.md
- ✅ Create RELEASE_NOTES.md
- ✅ Create MIGRATION_GUIDE.md
- ⏳ Run all verification commands
- ⏳ Fix any issues found

### Phase 3: GitHub Setup (After extraction complete)
- ⏳ Create repository: https://github.com/SuperInstance/tripartite-rs
- ⏳ Push code to GitHub
- ⏳ Configure GitHub Actions CI/CD
- ⏳ Test CI/CD passes on all platforms

### Phase 4: crates.io Publishing (After orchestrator approval)
- ⏳ Run `cargo publish --dry-run`
- ⏳ Fix any issues from dry run
- ⏳ Run `cargo publish` for real
- ⏳ Verify on crates.io and docs.rs

### Phase 5: SuperInstance Migration (After publish)
- ⏳ Update SuperInstance's Cargo.toml
- ⏳ Update all imports in synesis-core
- ⏳ Remove consensus code from synesis-core
- ⏳ Run `cargo test --workspace`
- ⏳ Verify all 250+ tests still pass
- ⏳ Commit and push changes

## Notes

- **DO NOT publish to crates.io until orchestrator approves**
- **GitHub repo can be created anytime** (can be private initially)
- **SuperInstance can use path dependency during development**:
  ```toml
  [dependencies]
  tripartite-rs = { path = "../tripartite-rs" }
  ```
- **Switch to crates.io dependency only after published**:
  ```toml
  [dependencies]
  tripartite-rs = "0.1"
  ```

## Rollback Plan

If publishing fails or causes issues:

1. **Yank the version** (if published):
   ```bash
   # Go to https://crates.io/crates/tripartite-rs
   # Click "Yank" button for v0.1.0
   ```

2. **Revert SuperInstance changes**:
   ```bash
   cd /mnt/c/claudesuperinstance
   git checkout main
   git reset --hard HEAD~1  # Undo migration commit
   ```

3. **Fix issues** and repeat publishing process

4. **Publish v0.1.1** with fixes:
   ```bash
   # Bump version in Cargo.toml to 0.1.1
   # Fix issues
   # cargo publish
   ```

## Success Criteria

Publishing is successful when:
- ✅ Crate appears on https://crates.io/crates/tripartite-rs
- ✅ Documentation builds on https://docs.rs/tripartite-rs
- ✅ Can install with `cargo add tripartite-rs`
- ✅ SuperInstance builds and runs with tripartite-rs dependency
- ✅ All 250+ SuperInstance tests still pass
- ✅ No compiler warnings in SuperInstance workspace

---

**Created**: 2026-01-08
**Agent**: 2.4 (Publishing & Integration Specialist)
**Status**: Ready for verification phase
