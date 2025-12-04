# Configuration

## Server Setup

```rust
use mcp_axum::McpServer;

let mut server = McpServer::new();
server.register_tool("tool", MyTool)?;
server.serve("0.0.0.0:8080").await?;
```

## Custom Config

Defaults: 30s timeouts, 10MB max body size.

```rust
use mcp_axum::{McpServer, ServerConfig};
use std::time::Duration;

let config = ServerConfig::new()
    .with_tool_timeout(Duration::from_secs(60))
    .with_resource_timeout(Duration::from_secs(30))
    .with_prompt_timeout(Duration::from_secs(10))
    .with_max_body_size(20 * 1024 * 1024);

let mut server = McpServer::with_config(config);
```

## Environment Variables

```rust
use std::env;

let api_key = env::var("API_KEY")?;
```

## Client Config

### Claude Desktop

`~/Library/Application Support/Claude/claude_desktop_config.json` (macOS)  
`%APPDATA%\Claude\claude_desktop_config.json` (Windows)

```json
{
  "mcpServers": {
    "server": {
      "type": "http",
      "url": "http://localhost:8080"
    }
  }
}
```

### With Auth

```json
{
  "mcpServers": {
    "server": {
      "type": "http",
      "url": "https://mcp.example.com",
      "headers": {
        "Authorization": "Bearer TOKEN"
      }
    }
  }
}
```

## Deployment

```bash
cargo build --release
./target/release/my-server
```

Docker:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/my-server /usr/local/bin/
CMD ["my-server"]
```

## Custom Middleware

```rust
use axum::middleware;

let app = server.router()
    .layer(middleware::from_fn(auth_middleware));
```

See `examples/auth_middleware.rs` for auth example.
