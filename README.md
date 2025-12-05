# axum-mcp

HTTP transport for Model Context Protocol servers.

## What

MCP server framework using HTTP instead of stdio. Implements MCP spec over REST endpoints.

## Why HTTP

- Remote access (server/client on different machines)
- Multiple concurrent clients
- Standard HTTP infrastructure (load balancers, proxies, monitoring)
- Cloud deployments

Not for stdio-based clients (e.g., Claude Desktop local mode).

## Install

```toml
[dependencies]
axum-mcp = "0.2"
```

## Example

```rust
use axum_mcp::{extract_string, McpServer, Tool};
use async_trait::async_trait;
use serde_json::Value;

struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn description(&self) -> &str { "Echo input" }
    
    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {"text": {"type": "string"}},
            "required": ["text"]
        })
    }
    
    async fn call(&self, args: &Value) -> Result<Value, String> {
        let text = extract_string(args, "text")?;
        Ok(serde_json::json!({ "echoed": text }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = McpServer::new()
        .tool("echo", EchoTool)?;
    server.serve("127.0.0.1:8080").await?;
    Ok(())
}
```

## API

- `GET /health` - Health check
- `GET /tools/list` - List tools
- `POST /tools/call` - Execute tool
- `GET /resources/list` - List resources
- `POST /resources/read` - Read resource
- `GET /prompts/list` - List prompts
- `POST /prompts/get` - Render prompt

## Traits

### Tool

```rust
#[async_trait]
trait Tool: Send + Sync {
    fn description(&self) -> &str;
    fn schema(&self) -> Value;  // JSON Schema
    async fn call(&self, arguments: &Value) -> Result<Value, String>;
}
```

### Resource

```rust
#[async_trait]
trait Resource: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn mime_type(&self) -> &str;
    async fn read(&self) -> Result<String, String>;
}
```

### Prompt

```rust
#[async_trait]
trait Prompt: Send + Sync {
    fn description(&self) -> &str;
    fn arguments(&self) -> Value;  // JSON Schema
    async fn render(&self, arguments: &Value) -> Result<String, String>;
}
```

## Utilities

Argument extraction helpers:

```rust
use axum_mcp::{extract_string, extract_string_opt, extract_number, extract_number_opt,
                extract_integer, extract_integer_opt, extract_bool, extract_bool_opt};

let text = extract_string(args, "text")?;
let limit = extract_integer_opt(args, "limit").unwrap_or(10);
```

Testing (with `testing` feature):

```rust
use axum_mcp::test_tool;

let result = test_tool(&tool, json!({"text": "hello"})).await?;
```

Custom middleware:

```rust
use axum::middleware;

let app = server.router()
    .layer(middleware::from_fn(auth_middleware));
```

Schema from docstrings (optional utility):

```rust
use axum_mcp::schema::extract_schema_from_docstring;

fn schema(&self) -> Value {
    extract_schema_from_docstring(r#"
        # Arguments
        * `text` - Input text (type: string)
    "#)
}
```

## Configuration

Defaults: 30s timeouts, 10MB max body size.

```rust
use axum_mcp::{McpServer, ServerConfig};
use std::time::Duration;

let config = ServerConfig::new()
    .with_tool_timeout(Duration::from_secs(60))
    .with_resource_timeout(Duration::from_secs(30))
    .with_prompt_timeout(Duration::from_secs(10))
    .with_max_body_size(20 * 1024 * 1024);

let mut server = McpServer::with_config(config);
```

## Builder Pattern

```rust
let server = McpServer::new()
    .tool("echo", EchoTool)?
    .resource("file://data", FileResource)?
    .prompt("greeting", GreetingPrompt)?;
```

## Features

- Request timeouts (30s default)
- JSON Schema validation of tool arguments before execution
- Validates tool names, resource URIs, and prompt names per MCP spec
- Request logging with request IDs
- Graceful shutdown
- CORS enabled
- Request body size limits (10MB default)

## Error Handling

Errors return HTTP status codes:
- `400` - Bad request (invalid arguments, missing parameters, schema validation failed)
- `404` - Not found (tool/resource/prompt doesn't exist)
- `500` - Internal server error (tool/resource/prompt execution failed)

Error response format:
```json
{
  "code": 400,
  "message": "Missing required parameter 'text'",
  "details": null
}
```

## Limitations

- No procedural macros (implement traits manually)
- HTTP only (no stdio transport)
- Arguments use `serde_json::Value` (not type-safe)

## Client Config

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

## License

MIT OR Apache-2.0
