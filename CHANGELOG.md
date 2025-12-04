# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive test suite covering integration, edge cases, performance, and security
- GitHub Actions CI workflow with check, test, and e2e jobs
- Clippy and rustfmt configuration
- Graceful shutdown support
- Request timeouts for tools, resources, and prompts
- JSON Schema validation for tool arguments
- MCP specification-compliant input validation
- Request body size limits
- Request logging and tracing with request IDs
- CORS support enabled by default
- Comprehensive error handling with structured HTTP responses
- **Argument extraction utilities** - Helper functions to reduce boilerplate (`extract_string`, `extract_number`, etc.)
- **Testing utilities** - `test_tool()` helper for unit testing tools (with `testing` feature)
- **Authentication middleware example** - API key authentication pattern for production
- **Client integration guide** (CLIENTS.md) - Complete guide for non-Rust developers
- **Dockerfile** - Production deployment example
- Quick test guide in README with curl examples
- Configuration guide (CONFIGURATION.md) for deployment

### Changed
- **Improved registration API ergonomics** - `register_tool()`, `register_resource()`, and `register_prompt()` now accept `impl Into<String>` for names and handle `Arc` internally, eliminating `.to_string()` and `Arc::new()` boilerplate
- Improved error messages and validation
- Enhanced documentation
- Removed unused rate limiting configuration (simplified config to only include implemented features)
- Removed placeholder macros.rs file (macros will be in separate crate when implemented)
- Removed duplicate tower dependency from dev-dependencies
- Updated CHANGELOG to use accurate test suite description

## [0.1.0] - 2025-01-XX

### Added
- Initial release
- HTTP-based MCP server framework
- Trait-based handlers for tools, resources, and prompts
- Docstring schema extraction (optional)
- Built on `axum` for routing and async support
- Configuration options for timeouts, limits, and behavior

