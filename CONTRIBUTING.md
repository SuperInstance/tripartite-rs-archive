# Contributing to tripartite-rs

Thank you for your interest in contributing to tripartite-rs! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow:
- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy toward other contributors

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check the existing issues to avoid duplicates. When creating a bug report, include:
- A clear and descriptive title
- Steps to reproduce the issue
- Expected behavior vs. actual behavior
- Environment details (OS, Rust version)
- Any relevant error messages or logs

### Suggesting Enhancements

Enhancement suggestions are welcome! Please:
- Use a clear and descriptive title
- Provide a detailed explanation of the feature
- Explain why this feature would be useful
- Include examples of how the feature would be used

### Pull Requests

1. Fork the repository
2. Create a branch for your work (`git checkout -b feature/amazing-feature`)
3. Make your changes following our coding standards
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Ensure no clippy warnings (`cargo clippy -- -D warnings`)
7. Format your code (`cargo fmt`)
8. Commit your changes with clear messages
9. Push to your fork and submit a pull request

## Development Setup

```bash
# Clone your fork
git clone https://github.com/your-username/tripartite-rs.git
cd tripartite-rs

# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build

# Run tests
cargo test

# Run examples
cargo run --example basic
cargo run --example multi_round
cargo run --example weighted_voting
cargo run --example veto
cargo run --example ml_ensembling

# Run benchmarks
cargo bench
```

## Coding Standards

### Rust Style

- Follow standard Rust style guidelines (`cargo fmt`)
- Use meaningful variable and function names
- Add doc comments to all public APIs
- Keep functions focused and concise
- Prefer `Result` over `panic!` for error handling

### Documentation

- Document all public APIs with `///` doc comments
- Include examples in documentation
- Update README.md if adding user-facing features
- Keep CHANGELOG.md updated for significant changes

### Testing

- Write unit tests for new functionality
- Maintain test coverage above 80%
- Use descriptive test names
- Test edge cases and error conditions

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
feat: add support for custom voting strategies
fix: resolve deadlock in multi-round consensus
docs: update README with new examples
test: add integration tests for veto mechanism
refactor: simplify agent trait implementation
```

## Project Structure

```
tripartite-rs/
├── src/
│   ├── agent.rs       # Agent trait and related types
│   ├── consensus.rs   # Consensus engine implementation
│   ├── error.rs       # Error types
│   ├── manifest.rs    # Agent-to-agent communication protocol
│   └── lib.rs         # Library entry point
├── examples/          # Example programs
├── benches/           # Performance benchmarks
├── tests/             # Integration tests
└── docs/              # Additional documentation
```

## License

By contributing to tripartite-rs, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.

## Questions?

Feel free to open an issue for any questions about contributing!
