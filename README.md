# mcp-axum

HTTP-based MCP server framework for Rust, built on `axum`.

## What is this?

A framework for building [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) servers using HTTP transport instead of stdio. MCP enables AI agents to discover and use tools, resources, and prompts from external systems.

## Use Cases

**When to use this:**

- **Remote MCP servers**: Deploy MCP capabilities as web services accessible over HTTP
- **Multi-client scenarios**: Multiple AI agents/clients connecting to the same server
- **Cloud deployments**: Host MCP servers in cloud environments (AWS, GCP, etc.)
- **HTTP-first infrastructure**: Integrate with existing HTTP-based systems (load balancers, API gateways, reverse proxies)
- **Centralized tooling**: Manage tools/resources/prompts in one place, accessible to multiple clients
- **Web-based AI applications**: Build web UIs that connect to MCP servers over HTTP

**Concrete examples:**
- Company-wide AI assistant with shared tools (database queries, internal APIs)
- SaaS platform exposing MCP endpoints for customer AI integrations
- Microservices architecture where each service exposes MCP capabilities
- Development tools accessible via HTTP (code analysis, testing, deployment)

**Not for you if:**
- You need stdio transport (e.g., for Claude Desktop local integration)
- You want procedural macros (not yet implemented - you implement traits manually)
- You only need a simple REST API (this adds MCP-specific structure you may not need)

## Quick Start

```rust
use mcp_axum::{extract_string, McpServer, Tool};
use async_trait::async_trait;
use serde_json::Value;

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
        // Use helper function instead of manual extraction
        let text = extract_string(arguments, "text")?;
        Ok(serde_json::json!({ "echoed": text }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let mut server = McpServer::new();
    server.register_tool("echo", EchoTool)?;  // No .to_string() or Arc::new() needed!
    server.serve("127.0.0.1:8080").await?;
    
    Ok(())
}
```

**Quick test:**
```bash
# Start server
cargo run --example basic_server &

# Test health
curl http://localhost:8080/health

# List tools
curl http://localhost:8080/tools/list

# Call tool
curl -X POST http://localhost:8080/tools/call \
  -H 'Content-Type: application/json' \
  -d '{"name":"echo","arguments":{"text":"hello"}}'
```

## API Endpoints

- `GET /health` - Health check
- `GET /tools/list` - List all tools
- `POST /tools/call` - Execute a tool
- `GET /resources/list` - List all resources
- `POST /resources/read` - Read a resource
- `GET /prompts/list` - List all prompts
- `POST /prompts/get` - Get a rendered prompt

## Configuration

You can customize timeouts and limits:

```rust
use mcp_axum::{McpServer, ServerConfig};
use std::time::Duration;

let config = ServerConfig::new()
    .with_tool_timeout(Duration::from_secs(60))
    .with_max_body_size(20 * 1024 * 1024);

let mut server = McpServer::with_config(config);
```

## What's included

- HTTP endpoints for tools, resources, and prompts
- Request timeouts (default: 30s)
- JSON Schema validation of tool arguments
- Input validation per MCP spec
- Request logging with request IDs
- Graceful shutdown support
- CORS enabled by default
- **Argument extraction helpers** - Reduce boilerplate (`extract_string`, `extract_number`, etc.)
- **Testing utilities** (with `testing` feature) - Test tools without starting a server

## Limitations

- **No procedural macros yet** - You implement `Tool`, `Resource`, and `Prompt` traits manually
- **HTTP only** - Not compatible with stdio-based MCP clients

## Examples

See the [`examples/`](examples/) directory for:
- Basic server setup
- **Utility functions** - Using argument extraction helpers to reduce boilerplate
- Advanced server with multiple tools/resources/prompts
- File system resources
- Database tools
- API integration patterns
- Configuration options
- Graceful shutdown
- Production deployment example
- **Authentication middleware** - API key authentication example

**Quick client configuration example:**
```json
{
  "mcpServers": {
    "my-server": {
      "type": "http",
      "url": "http://localhost:8080"
    }
  }
}
```

See [CONFIGURATION.md](CONFIGURATION.md) for complete setup guide.

## For Non-Rust Developers

**This library builds Rust MCP servers, but any language can connect to them!**

HTTP MCP servers expose standard REST endpoints that work with any HTTP client:

- **Python**: Use `httpx` or `requests` - see [CLIENTS.md](CLIENTS.md)
- **JavaScript/TypeScript**: Use `fetch` or `axios` - see [CLIENTS.md](CLIENTS.md)
- **Any language**: Use your standard HTTP client library

The server is language-agnostic - it's just JSON over HTTP. See [CLIENTS.md](CLIENTS.md) for complete client integration guide with examples in multiple languages.

## Why HTTP instead of stdio?

MCP supports two transports:

- **stdio**: Client launches server as subprocess, communicates via stdin/stdout. Best for local, single-client scenarios (e.g., Claude Desktop).
- **HTTP/SSE**: Server runs independently, handles multiple clients. Best for remote, multi-client, cloud deployments.

This library implements **HTTP transport**, enabling:
- Remote access (server can be on different machine/network)
- Multiple concurrent clients
- Standard HTTP infrastructure (load balancers, auth, monitoring)
- Cloud-native deployment patterns

## Comparison

| Library | Transport | Best For |
|---------|-----------|----------|
| `mcp-axum` | HTTP/SSE | Web services, cloud deployments, multi-client |
| `rust-mcp-sdk` | stdio | CLI tools, Claude Desktop, local integrations |
| `rmcp` | stdio | Direct JSON-RPC handling, local use |

## Installation

```toml
[dependencies]
mcp-axum = "0.2"

# For testing utilities
mcp-axum = { version = "0.2", features = ["testing"] }
```

## Documentation

- [API Documentation](https://docs.rs/mcp-axum)
- [Configuration Guide](CONFIGURATION.md) - How to configure and deploy HTTP-based MCP servers
- [Client Integration Guide](CLIENTS.md) - **For non-Rust developers** - Connect to MCP servers from any language
- [Examples](examples/)
- [Model Context Protocol](https://modelcontextprotocol.io/)

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
