//! Example demonstrating the builder pattern for server registration.
//!
//! This shows how to use the chainable `.tool()`, `.resource()`, and `.prompt()` methods
//! for a more ergonomic server setup.

use async_trait::async_trait;
use axum_mcp::{extract_string, McpServer, Prompt, Resource, Tool};
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
        let text = extract_string(arguments, "text")?;
        Ok(serde_json::json!({ "echoed": text }))
    }
}

struct HelloResource;

#[async_trait]
impl Resource for HelloResource {
    fn name(&self) -> &str {
        "Hello Resource"
    }

    fn description(&self) -> &str {
        "A simple hello resource"
    }

    fn mime_type(&self) -> &str {
        "text/plain"
    }

    async fn read(&self) -> Result<String, String> {
        Ok("Hello, World!".to_string())
    }
}

struct GreetingPrompt;

#[async_trait]
impl Prompt for GreetingPrompt {
    fn description(&self) -> &str {
        "Generate a greeting"
    }

    fn arguments(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name to greet",
                    "default": "World"
                }
            }
        })
    }

    async fn render(&self, arguments: &Value) -> Result<String, String> {
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");
        Ok(format!("Hello, {}!", name))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Use builder pattern for chainable registration
    let server = McpServer::new()
        .tool("echo", EchoTool)?
        .resource("hello://world", HelloResource)?
        .prompt("greeting", GreetingPrompt)?;

    println!("Server configured using builder pattern!");
    println!("Starting server on http://127.0.0.1:8080");
    server.serve("127.0.0.1:8080").await?;

    Ok(())
}
