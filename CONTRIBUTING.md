# Contributing

## Setup

```bash
git clone https://github.com/arclabs561/axum-mcp.git
cd axum-mcp
cargo test
```

## Checks

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Process

1. Fork and branch
2. Make changes
3. Run checks
4. Update docs if needed
5. Update CHANGELOG.md
6. Open PR

## Style

- Rust standard formatting
- Clippy warnings as errors
- Document public APIs
- Add tests for new features
