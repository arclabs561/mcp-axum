//! Basic MCP server example.
//!
//! This example demonstrates how to create a simple MCP server with tools,
//! resources, and prompts.

use async_trait::async_trait;
use mcp_axum::{extract_string, extract_string_opt, McpServer, Prompt, Resource, Tool};
use serde_json::Value;

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
        let text = extract_string(arguments, "text")?;
        Ok(serde_json::json!({
            "echoed": text
        }))
    }
}

/// Example resource: Hello
struct HelloResource;

#[async_trait]
impl Resource for HelloResource {
    fn name(&self) -> &str {
        "Hello World Resource"
    }

    fn description(&self) -> &str {
        "A simple hello world resource"
    }

    fn mime_type(&self) -> &str {
        "text/plain"
    }

    async fn read(&self) -> Result<String, String> {
        Ok("Hello, World!".to_string())
    }
}

/// Example prompt: Greeting
struct GreetingPrompt;

#[async_trait]
impl Prompt for GreetingPrompt {
    fn description(&self) -> &str {
        "Generate a greeting message"
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
        let name = extract_string_opt(arguments, "name").unwrap_or_else(|| "World".to_string());
        Ok(format!("Hello, {}!", name))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create server
    let mut server = McpServer::new();

    // Register tools
    server.register_tool("echo", EchoTool)?;

    // Register resources
    server.register_resource("hello://world", HelloResource)?;

    // Register prompts
    server.register_prompt("greeting", GreetingPrompt)?;

    // Start server
    server.serve("127.0.0.1:8080").await?;

    Ok(())
}
