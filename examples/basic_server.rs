//! Basic MCP server example.
//!
//! This example demonstrates how to create a simple MCP server with tools,
//! resources, and prompts.

use mcp_axum::{McpServer, Tool, Resource, Prompt};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

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
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'text' parameter".to_string())?;
        
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
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");
        
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
    server.register_tool("echo".to_string(), Arc::new(EchoTool))?;

    // Register resources
    server.register_resource("hello://world".to_string(), Arc::new(HelloResource))?;
    
    // Register prompts
    server.register_prompt("greeting".to_string(), Arc::new(GreetingPrompt))?;

    // Start server
    server.serve("127.0.0.1:8080").await?;

    Ok(())
}

