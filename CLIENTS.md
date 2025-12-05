# Client Integration

HTTP API reference for connecting to axum-mcp servers.

## Endpoints

### Health

```http
GET /health
```

```json
{"status": "ok"}
```

### List Tools

```http
GET /tools/list
```

```json
{
  "tools": [{
    "name": "echo",
    "description": "Echo input",
    "inputSchema": {
      "type": "object",
      "properties": {"text": {"type": "string"}},
      "required": ["text"]
    }
  }]
}
```

### Call Tool

```http
POST /tools/call
Content-Type: application/json

{"name": "echo", "arguments": {"text": "hello"}}
```

```json
{
  "content": [{
    "type": "text",
    "text": "{\"echoed\":\"hello\"}"
  }]
}
```

### List Resources

```http
GET /resources/list
```

```json
{
  "resources": [{
    "uri": "file://data",
    "name": "Data",
    "description": "Data resource",
    "mimeType": "text/plain"
  }]
}
```

### Read Resource

```http
POST /resources/read
Content-Type: application/json

{"uri": "file://data"}
```

```json
{
  "contents": [{
    "uri": "file://data",
    "mimeType": "text/plain",
    "text": "content"
  }]
}
```

### List Prompts

```http
GET /prompts/list
```

```json
{
  "prompts": [{
    "name": "greeting",
    "description": "Greeting prompt",
    "arguments": {
      "type": "object",
      "properties": {
        "name": {
          "type": "string",
          "description": "Name"
        }
      }
    }
  }]
}
```

### Get Prompt

```http
POST /prompts/get
Content-Type: application/json

{"name": "greeting", "arguments": {"name": "Alice"}}
```

```json
{
  "messages": [{
    "role": "user",
    "content": {"type": "text", "text": "Hello, Alice!"}
  }]
}
```

## Examples

### Python

```python
import httpx

async def call_tool(name: str, args: dict):
    async with httpx.AsyncClient() as client:
        r = await client.post(
            "http://localhost:8080/tools/call",
            json={"name": name, "arguments": args}
        )
        return r.json()
```

### JavaScript

```javascript
async function callTool(name, args) {
  const r = await fetch('http://localhost:8080/tools/call', {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({name, arguments: args})
  });
  return r.json();
}
```

### cURL

```bash
curl -X POST http://localhost:8080/tools/call \
  -H 'Content-Type: application/json' \
  -d '{"name":"echo","arguments":{"text":"hello"}}'
```

## Status Codes

- `200` - Success
- `400` - Bad request
- `401` - Unauthorized
- `404` - Not found
- `500` - Server error

## Error Response

```json
{
  "code": 400,
  "message": "Missing required parameter 'text'",
  "details": null
}
```

## Auth

```bash
curl -H 'Authorization: Bearer TOKEN' \
  http://localhost:8080/tools/list
```
