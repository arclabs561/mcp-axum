# API/Interface/UX Design Analysis

## Current Design Review

### 1. Registration API (`McpServer`)

**Current:**
```rust
let mut server = McpServer::new();
server.register_tool("echo".to_string(), Arc::new(EchoTool))?;
server.register_resource("hello://world".to_string(), Arc::new(HelloResource))?;
server.register_prompt("greeting".to_string(), Arc::new(GreetingPrompt))?;
```

**Issues:**
1. **Repetitive `.to_string()`** - Every registration requires converting `&str` to `String`
2. **Arc boilerplate** - `Arc::new()` on every registration
3. **Mutable server** - Must use `mut` even though conceptually we're just building
4. **No builder pattern** - Can't chain registrations
5. **Inconsistent with Rust idioms** - Most Rust APIs accept `impl Into<String>` or `&str`

**Better design:**
```rust
// Option 1: Builder pattern (like axum Router)
let server = McpServer::new()
    .tool("echo", EchoTool)
    .resource("hello://world", HelloResource)
    .prompt("greeting", GreetingPrompt);

// Option 2: Accept &str and handle Arc internally
server.register_tool("echo", EchoTool)?;  // Arc handled internally

// Option 3: Macro for common case
mcp_server! {
    tool("echo", EchoTool);
    resource("hello://world", HelloResource);
    prompt("greeting", GreetingPrompt);
}
```

### 2. Trait Design (`Tool`, `Resource`, `Prompt`)

**Current:**
```rust
#[async_trait]
impl Tool for EchoTool {
    fn description(&self) -> &str { ... }
    fn schema(&self) -> Value { ... }
    async fn call(&self, arguments: &Value) -> Result<Value, String> { ... }
}
```

**Issues:**
1. **`&Value` everywhere** - No type safety, manual extraction
2. **`Result<Value, String>`** - String errors lose context, no structured errors
3. **`schema()` returns `Value`** - Could be a typed struct
4. **No default implementations** - Every tool must implement all methods

**Better design:**
```rust
// Option 1: Associated types for type safety
trait Tool {
    type Args: Deserialize;
    type Output: Serialize;
    type Error: Into<String>;
    
    fn description(&self) -> &str;
    fn schema(&self) -> Value { /* derive from Args */ }
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error>;
}

// Option 2: Structured errors
async fn call(&self, arguments: &Value) -> Result<Value, ToolError>;

// Option 3: Default implementations
trait Tool {
    fn description(&self) -> &str { "No description" }
    fn schema(&self) -> Value { /* derive from Args if possible */ }
}
```

### 3. Argument Extraction (`utils.rs`)

**Current:**
```rust
let text = extract_string(arguments, "text")?;
let limit = extract_integer_opt(arguments, "limit").unwrap_or(10);
```

**Issues:**
1. **No type inference** - Must specify function name (`extract_string` vs `extract_integer`)
2. **Repetitive** - Similar pattern for each type
3. **No validation** - Can't specify min/max, patterns, etc.
4. **Error messages** - Generic, not contextual

**Better design:**
```rust
// Option 1: Macro for type inference
let text: String = extract!(arguments, "text")?;
let limit: Option<u64> = extract!(arguments, "limit");

// Option 2: Builder pattern
let args = Arguments::from(arguments)
    .required::<String>("text")?
    .optional::<u64>("limit").unwrap_or(10);

// Option 3: Derive macro
#[derive(Deserialize)]
struct EchoArgs {
    text: String,
}
let args: EchoArgs = serde_json::from_value(arguments.clone())?;
```

### 4. Configuration (`ServerConfig`)

**Current:**
```rust
let config = ServerConfig::new()
    .with_tool_timeout(Duration::from_secs(60))
    .with_max_body_size(20 * 1024 * 1024);
```

**Good:** Builder pattern is ergonomic ✅

**Issues:**
1. **No validation** - Can set invalid values (negative timeouts, etc.)
2. **No documentation in code** - Must read docs to know defaults
3. **No environment variable support** - Must set programmatically

**Better design:**
```rust
// Option 1: Validation
impl ServerConfig {
    pub fn with_tool_timeout(mut self, timeout: Duration) -> Result<Self, ConfigError> {
        if timeout.is_zero() {
            return Err(ConfigError::InvalidTimeout);
        }
        self.tool_timeout = timeout;
        Ok(self)
    }
}

// Option 2: From environment
impl ServerConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Read from env vars
    }
}
```

### 5. Error Handling

**Current:**
```rust
async fn call(&self, arguments: &Value) -> Result<Value, String>;
```

**Issues:**
1. **`String` errors** - No structure, hard to handle programmatically
2. **No error context** - Can't distinguish error types
3. **No error codes** - Can't map to HTTP status codes easily

**Better design:**
```rust
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
    
    #[error("Invalid parameter '{param}': expected {expected}, got {got}")]
    InvalidType { param: String, expected: String, got: String },
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

async fn call(&self, arguments: &Value) -> Result<Value, ToolError>;
```

### 6. Server Lifecycle

**Current:**
```rust
let mut server = McpServer::new();
server.register_tool("echo", Arc::new(EchoTool))?;
server.serve("127.0.0.1:8080").await?;
```

**Issues:**
1. **Consumes server** - Can't reuse after `serve()`
2. **No router access** - Must use `router()` separately for middleware
3. **Inconsistent** - `serve()` vs `router()` have different patterns

**Better design:**
```rust
// Option 1: Separate builder and server
let builder = McpServer::builder()
    .tool("echo", EchoTool)
    .build()?;
builder.serve("127.0.0.1:8080").await?;

// Option 2: Router is primary, serve is convenience
let router = McpServer::new()
    .tool("echo", EchoTool)
    .into_router();
axum::serve(listener, router).await?;
```

## Design Principles Assessment

### ✅ Good Design Decisions

1. **Builder pattern for config** - Follows Rust idioms
2. **Trait-based handlers** - Flexible, extensible
3. **Async throughout** - Modern Rust async/await
4. **Error types** - Using `thiserror` for structured errors
5. **Validation** - MCP spec compliance built-in

### ⚠️ Areas for Improvement

1. **Registration ergonomics** - Too much boilerplate
2. **Type safety** - Too much `Value`, not enough types
3. **Error handling** - `String` errors lose structure
4. **Consistency** - Some APIs use builders, others don't
5. **Discoverability** - Hard to know what's available

## Recommended Improvements (Priority Order)

### P0: High Impact, Low Effort

1. **Accept `&str` in registration** - Remove `.to_string()` requirement
   ```rust
   pub fn register_tool(&mut self, name: impl Into<String>, tool: Arc<dyn Tool>)
   ```

2. **Handle Arc internally** - Users shouldn't need to wrap
   ```rust
   pub fn register_tool<T: Tool>(&mut self, name: impl Into<String>, tool: T)
   ```

3. **Better error types** - Replace `String` with structured errors
   ```rust
   pub enum ToolError { MissingParameter(String), ... }
   ```

### P1: Medium Impact, Medium Effort

4. **Builder pattern for registration** - Chainable API
   ```rust
   McpServer::new()
       .tool("echo", EchoTool)
       .resource("hello://world", HelloResource)
   ```

5. **Macro for argument extraction** - Type inference
   ```rust
   let text: String = extract!(arguments, "text")?;
   ```

6. **Derive support** - Generate schema from types
   ```rust
   #[derive(Deserialize, ToolArgs)]
   struct EchoArgs { text: String }
   ```

### P2: High Impact, High Effort

7. **Type-safe tools** - Associated types for args/output
8. **Procedural macros** - `#[mcp_tool]` attribute
9. **Environment config** - `ServerConfig::from_env()`

## Comparison with Axum's Design

**What Axum does well:**
- Builder pattern throughout (`Router::new().route(...).layer(...)`)
- Type-safe extractors (`Json<T>`, `Query<T>`, etc.)
- Flexible error handling (`Result<T, E>` where `E: IntoResponse`)
- Composable middleware
- Clear separation of concerns

**What we should adopt:**
- Builder pattern for server setup
- Type-safe argument extraction (like `Json<T>`)
- Better error handling (structured, composable)
- More composable API

## UX Pain Points Summary

1. **Too much boilerplate** - `.to_string()`, `Arc::new()` everywhere
2. **No type safety** - Everything is `Value`
3. **String errors** - Hard to handle programmatically
4. **Inconsistent patterns** - Some builders, some not
5. **Hard to discover** - No IDE autocomplete hints for common patterns

## Recommended Next Steps

1. **Quick wins** - Accept `&str`, handle `Arc` internally
2. **Better errors** - Structured error types
3. **Builder pattern** - Chainable registration
4. **Type safety** - Derive macros for args
5. **Documentation** - More examples, better docs

