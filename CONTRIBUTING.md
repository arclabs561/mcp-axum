# Contributing to mcp-axum

Thank you for your interest in contributing to `mcp-axum`! This document provides guidelines and instructions for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/mcp-axum.git`
3. Create a branch: `git checkout -b your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Run clippy: `cargo clippy --all-targets --all-features -- -D warnings`
7. Format code: `cargo fmt --all`
8. Commit your changes: `git commit -m "Add your feature"`
9. Push to your fork: `git push origin your-feature-name`
10. Open a pull request

## Development Setup

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/arclabs561/mcp-axum.git
cd mcp-axum

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all

# Build documentation
cargo doc --open
```

## Code Style

- Follow Rust standard formatting (enforced by `cargo fmt`)
- Follow clippy suggestions (enforced in CI)
- Write documentation for all public APIs
- Add tests for new features
- Keep functions focused and small
- Use meaningful variable and function names

## Testing

- Add tests for all new features
- Ensure all tests pass: `cargo test`
- Run integration tests: `cargo test --test integration_test`
- Test examples build: `cargo build --examples`

## Pull Request Process

1. Ensure all tests pass
2. Ensure clippy passes with no warnings
3. Ensure code is formatted
4. Update documentation if needed
5. Update CHANGELOG.md if adding features or fixing bugs
6. Write a clear description of your changes

## Commit Messages

Use clear, descriptive commit messages:

- `Add feature X`
- `Fix bug in Y`
- `Update documentation for Z`
- `Refactor module A`

## Questions?

Feel free to open an issue for questions or discussions about potential contributions.

