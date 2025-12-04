# Client Integration Guide

This guide is for **non-Rust developers** who want to connect to HTTP-based MCP servers built with `mcp-axum`, or understand how the HTTP API works.

## For Non-Rust Developers

Even though this library is for building MCP servers in Rust, **any language can connect to HTTP MCP servers** as a client. The server exposes standard HTTP endpoints that any HTTP client can use.

## HTTP API Reference

All endpoints use standard HTTP methods and JSON payloads.

### Base URL
```
http://localhost:8080  (or your server URL)
```

### Endpoints

#### Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "ok"
}
```

#### List Tools
```http
GET /tools/list
```

**Response:**
```json
{
  "tools": [
    {
      "name": "echo",
      "description": "Echo back the input text",
      "inputSchema": {
        "type": "object",
        "properties": {
          "text": {
            "type": "string",
            "description": "Text to echo back"
          }
        },
        "required": ["text"]
      }
    }
  ]
}
```

#### Call Tool
```http
POST /tools/call
Content-Type: application/json

{
  "name": "echo",
  "arguments": {
    "text": "hello"
  }
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"echoed\":\"hello\"}"
    }
  ]
}
```

#### List Resources
```http
GET /resources/list
```

**Response:**
```json
{
  "resources": [
    {
      "uri": "file://example.txt",
      "name": "Example File",
      "description": "An example resource",
      "mimeType": "text/plain"
    }
  ]
}
```

#### Read Resource
```http
POST /resources/read
Content-Type: application/json

{
  "uri": "file://example.txt"
}
```

**Response:**
```json
{
  "contents": [
    {
      "uri": "file://example.txt",
      "mimeType": "text/plain",
      "text": "File contents here"
    }
  ]
}
```

#### List Prompts
```http
GET /prompts/list
```

**Response:**
```json
{
  "prompts": [
    {
      "name": "greeting",
      "description": "Generate a greeting message",
      "arguments": [
        {
          "name": "name",
          "description": "Name to greet",
          "required": false
        }
      ]
    }
  ]
}
```

#### Get Prompt
```http
POST /prompts/get
Content-Type: application/json

{
  "name": "greeting",
  "arguments": {
    "name": "Alice"
  }
}
```

**Response:**
```json
{
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "Hello, Alice!"
      }
    }
  ]
}
```

## Client Libraries by Language

### Python

Use the official MCP Python SDK:

```python
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

# For HTTP transport, use requests or httpx directly
import httpx

async def call_tool(tool_name: str, arguments: dict):
    async with httpx.AsyncClient() as client:
        response = await client.post(
            "http://localhost:8080/tools/call",
            json={"name": tool_name, "arguments": arguments}
        )
        return response.json()

# Example
result = await call_tool("echo", {"text": "hello"})
print(result)
```

### JavaScript/TypeScript

```typescript
async function callTool(toolName: string, arguments: Record<string, any>) {
  const response = await fetch('http://localhost:8080/tools/call', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      name: toolName,
      arguments: arguments
    })
  });
  return await response.json();
}

// Example
const result = await callTool('echo', { text: 'hello' });
console.log(result);
```

### cURL (Any Language)

```bash
# List tools
curl http://localhost:8080/tools/list

# Call a tool
curl -X POST http://localhost:8080/tools/call \
  -H 'Content-Type: application/json' \
  -d '{"name":"echo","arguments":{"text":"hello"}}'
```

## Authentication

If the server uses authentication (see `examples/auth_middleware.rs`), include credentials:

```bash
# With API key
curl -H 'Authorization: Bearer YOUR_API_KEY' \
  http://localhost:8080/tools/list
```

```python
# Python example
headers = {'Authorization': 'Bearer YOUR_API_KEY'}
response = requests.get('http://localhost:8080/tools/list', headers=headers)
```

```typescript
// TypeScript example
const response = await fetch('http://localhost:8080/tools/list', {
  headers: {
    'Authorization': 'Bearer YOUR_API_KEY'
  }
});
```

## Error Handling

All endpoints return standard HTTP status codes:

- `200 OK` - Success
- `400 Bad Request` - Invalid request (missing parameters, validation errors)
- `401 Unauthorized` - Authentication required
- `404 Not Found` - Tool/resource/prompt not found
- `500 Internal Server Error` - Server error

Error responses include details:

```json
{
  "error": {
    "code": 400,
    "message": "Missing required parameter 'text'"
  }
}
```

## Complete Client Example (Python)

```python
import httpx
import asyncio

class MCPClient:
    def __init__(self, base_url: str, api_key: str = None):
        self.base_url = base_url
        self.headers = {}
        if api_key:
            self.headers['Authorization'] = f'Bearer {api_key}'
    
    async def list_tools(self):
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f'{self.base_url}/tools/list',
                headers=self.headers
            )
            response.raise_for_status()
            return response.json()
    
    async def call_tool(self, name: str, arguments: dict):
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f'{self.base_url}/tools/call',
                json={'name': name, 'arguments': arguments},
                headers=self.headers
            )
            response.raise_for_status()
            return response.json()

# Usage
async def main():
    client = MCPClient('http://localhost:8080', api_key='demo-key-123')
    
    # List available tools
    tools = await client.list_tools()
    print(f"Available tools: {[t['name'] for t in tools['tools']]}")
    
    # Call a tool
    result = await client.call_tool('echo', {'text': 'hello'})
    print(result)

asyncio.run(main())
```

## Integration with MCP Clients

### Claude Desktop

Configure in `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "my-rust-server": {
      "type": "http",
      "url": "http://localhost:8080",
      "headers": {
        "Authorization": "Bearer YOUR_API_KEY"
      }
    }
  }
}
```

### VS Code / Cursor

Create `mcp.json`:

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

## Testing Your Server

Even if you don't know Rust, you can test any HTTP MCP server:

```bash
# Health check
curl http://localhost:8080/health

# List tools
curl http://localhost:8080/tools/list | jq

# Call a tool
curl -X POST http://localhost:8080/tools/call \
  -H 'Content-Type: application/json' \
  -d '{"name":"echo","arguments":{"text":"test"}}' | jq
```

## Summary

**Key Point**: This library builds Rust MCP servers, but **any language can connect to them** via HTTP. The server exposes standard REST endpoints that work with any HTTP client library.

- **Python**: Use `httpx` or `requests`
- **JavaScript/TypeScript**: Use `fetch` or `axios`
- **Go**: Use `net/http`
- **Java**: Use `HttpClient` or `OkHttp`
- **Any language**: Use your standard HTTP client library

The MCP protocol over HTTP is language-agnostic - it's just JSON over HTTP.

