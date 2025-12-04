# mcp-axum

HTTP-based MCP server framework for Rust, built on `axum`.

## Overview

`mcp-axum` provides a simple way to build MCP servers in Rust:

- HTTP transport (unlike stdio-based alternatives)
- Trait-based handlers for tools, resources, and prompts
- Docstring schema extraction (optional)
- Built on `axum` for routing and async support

## Motivation

Most Rust MCP libraries use stdio transport (JSON-RPC over stdin/stdout), which works for CLI tools but requires manual protocol handling.

`mcp-axum` uses HTTP instead, making it easier to deploy as a web service and integrate with existing infrastructure. The docstring parser can extract JSON Schema from comments, though manual schema definitions are also supported.

## Model Context Protocol (MCP)

MCP is a protocol for AI agents to interact with external tools and data sources. It enables:

- **Tools**: Functions that agents can call (e.g., "search_arxiv", "read_file")
- **Resources**: Data sources agents can access (e.g., "file://...", "arxiv://...")
- **Prompts**: Template prompts for common tasks

## Installation

```toml
[dependencies]
mcp-axum = "0.1"
```

## Quick Start

```rust
use mcp_axum::{McpServer, Tool};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

/// Example tool: Echo
struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn description(&self) -> &str {
        "Echo back the input text"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "Text to echo back"
                }
            },
            "required": ["text"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'text' parameter".to_string())?;
        
        Ok(serde_json::json!({ "echoed": text }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let mut server = McpServer::new();
    server.register_tool("echo".to_string(), Arc::new(EchoTool))?;
    server.serve("127.0.0.1:8080").await?;
    
    Ok(())
}
```

See the [`examples/`](examples/) directory for complete examples including tools, resources, and prompts.

## API Endpoints

The server exposes RESTful endpoints:

- `GET /health` - Health check
- `GET /tools/list` - List all available tools
- `POST /tools/call` - Execute a tool with arguments
- `GET /resources/list` - List all available resources
- `POST /resources/read` - Read a resource by URI
- `GET /prompts/list` - List all available prompts
- `POST /prompts/get` - Get a rendered prompt with arguments

## Features

- Tools, resources, and prompts
- HTTP REST API endpoints
- Docstring schema extraction (optional)
- Built on `axum` for routing and middleware
- CORS enabled by default
- Error handling with HTTP status codes

## Status

Trait-based implementation. Procedural macros planned for future release.

## Documentation

- [API Documentation](https://docs.rs/mcp-axum)
- [Examples](examples/) - Complete working examples
- [Model Context Protocol](https://modelcontextprotocol.io/) - Official MCP documentation

## Comparison

| Library | Transport | Use Case |
|---------|-----------|----------|
| `mcp-axum` | HTTP | Web services, HTTP integration |
| `rust-mcp-sdk` | stdio | CLI tools, stdio-based servers |
| `rmcp` | stdio | Direct JSON-RPC handling |

Use `mcp-axum` for HTTP services. Use stdio-based libraries for CLI tools (e.g., Claude Desktop integration).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

