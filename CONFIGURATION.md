# Configuration Guide

This guide explains how to configure and use HTTP-based MCP servers built with `mcp-axum`.

## Quick Reference

**For stdio MCP servers** (traditional):
- Client launches server as subprocess
- Configuration: `{ "command": "npx", "args": [...] }`

**For HTTP MCP servers** (this library):
- Server runs independently as web service
- Configuration: `{ "type": "http", "url": "http://..." }`

## How MCP Server Configuration Works

MCP servers can use two transport mechanisms:

1. **stdio**: Client launches server as subprocess (local, single-client)
2. **HTTP/SSE**: Server runs independently (remote, multi-client)

This library implements **HTTP transport**, which means:
- The server runs as a standalone web service
- Clients connect via HTTP using a URL
- Multiple clients can connect to the same server

## Server-Side Configuration

### Basic Server Setup

The server is configured in Rust code:

```rust
use mcp_axum::{McpServer, Tool};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = McpServer::new();
    
    // Register your tools, resources, prompts
    server.register_tool("my_tool".to_string(), Arc::new(MyTool))?;
    
    // Start the server
    server.serve("0.0.0.0:8080").await?;
    Ok(())
}
```

### Custom Configuration

```rust
use mcp_axum::{McpServer, ServerConfig};
use std::time::Duration;

let config = ServerConfig::new()
    .with_tool_timeout(Duration::from_secs(60))
    .with_max_body_size(20 * 1024 * 1024);

let mut server = McpServer::with_config(config);
```

### Environment Variables

For sensitive configuration (API keys, database URLs), use environment variables:

```rust
use std::env;

let api_key = env::var("MY_API_KEY")
    .expect("MY_API_KEY environment variable not set");

// Use in your tools/resources
```

### Deployment

Deploy as a standard web service:

```bash
# Build
cargo build --release

# Run
./target/release/my-mcp-server

# Or with environment variables
MY_API_KEY=secret ./target/release/my-mcp-server
```

**Docker example:**
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/my-mcp-server /usr/local/bin/
CMD ["my-mcp-server"]
```

## Client-Side Configuration

### Claude Desktop

For HTTP-based MCP servers, configure in `claude_desktop_config.json`:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "my-http-server": {
      "type": "http",
      "url": "http://localhost:8080"
    },
    "my-remote-server": {
      "type": "http",
      "url": "https://mcp.example.com",
      "headers": {
        "Authorization": "Bearer YOUR_API_KEY"
      }
    }
  }
}
```

### VS Code / Cursor

Create `mcp.json` in your workspace:

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

### Programmatic Client Configuration

If building a custom client, connect via HTTP:

```typescript
// TypeScript example
const response = await fetch('http://localhost:8080/tools/list');
const { tools } = await response.json();

// Call a tool
await fetch('http://localhost:8080/tools/call', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    name: 'my_tool',
    arguments: { param: 'value' }
  })
});
```

## Complete Example Workflow

### 1. Build and Deploy Server

```bash
# Build the server
cargo build --release

# Deploy to cloud (example: AWS ECS, GCP Cloud Run, etc.)
# Server runs on https://mcp.mycompany.com
```

### 2. Configure Client

**Claude Desktop config:**
```json
{
  "mcpServers": {
    "company-tools": {
      "type": "http",
      "url": "https://mcp.mycompany.com"
    }
  }
}
```

### 3. Client Discovers Capabilities

The client automatically:
1. Calls `GET /tools/list` to discover available tools
2. Calls `GET /resources/list` to discover available resources
3. Calls `GET /prompts/list` to discover available prompts

### 4. Client Uses Tools

When the AI agent needs to use a tool:
1. Client calls `POST /tools/call` with tool name and arguments
2. Server validates arguments against JSON Schema
3. Server executes tool and returns result
4. Client receives result and continues

## Authentication

For production deployments, add authentication:

### Option 1: API Key in Headers

```json
{
  "mcpServers": {
    "secure-server": {
      "type": "http",
      "url": "https://mcp.example.com",
      "headers": {
        "Authorization": "Bearer YOUR_API_KEY"
      }
    }
  }
}
```

### Option 2: Server-Side Auth Middleware

Add authentication middleware to your server:

```rust
use axum::middleware;

let app = router()
    .layer(middleware::from_fn(auth_middleware))
    // ... rest of routes
```

## Health Checks

Clients can check server health:

```bash
curl http://localhost:8080/health
```

Returns `200 OK` if server is running.

## Comparison: stdio vs HTTP Configuration

### stdio (Traditional MCP)

**Server**: Runs as subprocess launched by client  
**Configuration**: Client specifies command to run
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path"]
    }
  }
}
```

### HTTP (This Library)

**Server**: Runs independently as web service  
**Configuration**: Client specifies URL
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

## Production Considerations

1. **Use HTTPS**: Always use HTTPS in production
2. **Authentication**: Add API keys or OAuth
3. **Rate Limiting**: Implement rate limiting (not included in library)
4. **Monitoring**: Add logging and metrics
5. **Load Balancing**: Use load balancers for multiple instances
6. **Environment Variables**: Store secrets in environment variables, not code

## Troubleshooting

**Server won't start:**
- Check port is available: `lsof -i :8080`
- Check firewall rules
- Verify bind address (use `0.0.0.0` for remote access)

**Client can't connect:**
- Verify server is running: `curl http://localhost:8080/health`
- Check URL in client config matches server address
- For remote servers, ensure firewall allows connections

**Tools not appearing:**
- Check server logs for registration errors
- Verify tool names are valid (alphanumeric + underscores)
- Test endpoints directly: `curl http://localhost:8080/tools/list`

