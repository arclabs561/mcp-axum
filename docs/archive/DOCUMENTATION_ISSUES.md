# Documentation Issues: Overstated, Unclear, Showy

## Overstated Claims

### 1. "type-safe trait-based handlers" (lib.rs:6)
**Problem:** Misleading. Everything uses `serde_json::Value`, which is not type-safe. The traits are type-safe, but data handling is not.

**Current:**
```rust
//! It supports docstring-driven schema extraction and type-safe trait-based handlers.
```

**Should be:**
```rust
//! Trait-based handlers for tools, resources, and prompts. Arguments use `serde_json::Value`.
```

### 2. "Axum-like framework" (lib.rs:3)
**Problem:** Vague. What does "like" mean? It's built on axum, not similar to it.

**Current:**
```rust
//! Axum-like framework for building Model Context Protocol (MCP) servers with HTTP transport.
```

**Should be:**
```rust
//! MCP server framework built on axum with HTTP transport.
```

### 3. "ergonomic framework" (lib.rs:5)
**Problem:** Subjective marketing language. Not factual.

**Current:**
```rust
//! This crate provides an ergonomic framework for building MCP servers in Rust using `axum`.
```

**Should be:**
```rust
//! Framework for building MCP servers in Rust using `axum`.
```

## Poorly Documented / Unclear

### 4. "Optional docstring schema extraction" (lib.rs:65, README)
**Problem:** 
- Function exists in `src/schema.rs` but is never used in server code
- No example of how to use it
- No explanation of what "optional" means
- Users can't actually use this feature

**Current:**
```rust
//! - Optional docstring schema extraction
```

**Should be:**
- Remove from features list, OR
- Add example showing how to use `extract_schema_from_docstring()` in a tool's `schema()` method
- Document that it's a utility function, not integrated into the server

### 5. "MCP spec validation" (README:166)
**Problem:** Vague. Which parts of the spec? What does it validate?

**Current:**
```markdown
- MCP spec validation
```

**Should be:**
```markdown
- Validates tool names, resource URIs, and prompt names per MCP spec
```

### 6. CLIENTS.md - Prompt arguments format is wrong
**Problem:** Shows array format but implementation uses JSON Schema object.

**Current (CLIENTS.md:102-106):**
```json
"arguments": [{
  "name": "name",
  "description": "Name",
  "required": false
}]
```

**Actual format (from code):**
```json
"arguments": {
  "type": "object",
  "properties": {
    "name": {
      "type": "string",
      "description": "Name"
    }
  }
}
```

**Fix:** Update CLIENTS.md to show correct JSON Schema format.

### 7. README example inconsistency
**Problem:** Shows `mut server` with `register_tool()` but builder pattern is shown later. Inconsistent.

**Current (README:54):**
```rust
let mut server = McpServer::new();
server.register_tool("echo", EchoTool)?;
```

**Later (README:156):**
```rust
let server = McpServer::new()
    .tool("echo", EchoTool)?
```

**Should be:** Pick one style and use consistently, or show both clearly labeled.

## Showy / Unnecessary Language

### 8. Comment in example: "No .to_string() or Arc::new() needed!" (lib.rs:55)
**Problem:** Showy. Just show the code.

**Current:**
```rust
server.register_tool("echo", EchoTool)?;  // No .to_string() or Arc::new() needed!
```

**Should be:**
```rust
server.register_tool("echo", EchoTool)?;
```

### 9. "Built on axum" redundancy
**Problem:** Mentioned in title, description, and features. Redundant.

**Current:**
- Title: "Built on axum"
- Description: "built on axum"
- Features: "Built on `axum` for async HTTP handling"

**Should be:** Mention once, remove redundancy.

### 10. "RESTful API endpoints" (lib.rs:63)
**Problem:** "RESTful" is vague. Just say "HTTP endpoints" or "REST endpoints".

**Current:**
```rust
//! - HTTP transport with RESTful API endpoints
```

**Should be:**
```rust
//! - HTTP transport with REST endpoints
```

## Missing Critical Information

### 11. No explanation of error handling
**Problem:** README mentions "Error handling with HTTP status codes" but doesn't explain:
- What errors are returned?
- What status codes?
- Error response format?

**Should add:**
```markdown
## Error Handling

Errors return HTTP status codes:
- `400` - Bad request (invalid arguments, missing parameters)
- `404` - Not found (tool/resource/prompt doesn't exist)
- `500` - Internal server error (tool execution failed)

Error response format:
```json
{
  "code": 400,
  "message": "Missing required parameter 'text'",
  "details": null
}
```
```

### 12. No explanation of response formats
**Problem:** README lists endpoints but doesn't show response formats. Users have to guess.

**Should add:** Response format examples for each endpoint, or link to CLIENTS.md.

### 13. "JSON Schema validation" - what gets validated?
**Problem:** Says "JSON Schema validation" but doesn't explain:
- What gets validated? (Tool arguments)
- When does it fail? (Invalid types, missing required fields)
- What error is returned?

**Should clarify:**
```markdown
- JSON Schema validation of tool arguments before execution
```

## Recommendations

### High Priority Fixes ✅

1. ✅ Remove "type-safe" claim (it's not type-safe) - FIXED
2. ✅ Fix CLIENTS.md prompt arguments format - FIXED
3. ✅ Remove or document "docstring schema extraction" - DOCUMENTED with example
4. ✅ Clarify "MCP spec validation" - what exactly is validated - FIXED
5. ✅ Remove showy comments from examples - FIXED
6. ✅ Make README example consistent (pick one style) - FIXED (uses builder pattern)

### Medium Priority ✅

7. ✅ Add error handling section to README - FIXED
8. ⚠️ Add response format examples - PARTIAL (CLIENTS.md has them, README links to it)
9. ✅ Clarify "JSON Schema validation" - what and when - FIXED
10. ✅ Remove "ergonomic" and "Axum-like" vague language - FIXED

### Low Priority ✅

11. ✅ Reduce redundancy ("built on axum" mentioned 3 times) - FIXED (removed from README title, Cargo.toml)
12. ✅ Simplify "RESTful" to "REST" - FIXED

