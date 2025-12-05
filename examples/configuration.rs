//! Example demonstrating server configuration options.
//!
//! This example shows how to:
//! - Configure timeouts
//! - Set request body size limits
//! - Configure timeouts and body size limits
//! - Access and modify configuration

use async_trait::async_trait;
use axum_mcp::{McpServer, ServerConfig, Tool};
use serde_json::{json, Value};
use std::time::Duration;

struct ConfigurableTool;

#[async_trait]
impl Tool for ConfigurableTool {
    fn description(&self) -> &str {
        "A tool that demonstrates configuration"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "A message"
                }
            },
            "required": ["message"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'message' parameter".to_string())?;

        Ok(json!({ "echoed": message }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Create configuration with custom values
    let config = ServerConfig::new()
        // Set tool execution timeout to 60 seconds
        .with_tool_timeout(Duration::from_secs(60))
        // Set resource read timeout to 45 seconds
        .with_resource_timeout(Duration::from_secs(45))
        // Set maximum request body size to 20MB
        .with_max_body_size(20 * 1024 * 1024);

    println!("Server Configuration:");
    println!("  Tool timeout: {:?}", config.tool_timeout);
    println!("  Resource timeout: {:?}", config.resource_timeout);
    println!("  Max body size: {} bytes", config.max_body_size);

    // Create server with custom configuration
    let mut server = McpServer::with_config(config);

    // You can also modify configuration after creation
    server.config_mut().tool_timeout = Duration::from_secs(90);
    println!(
        "\nUpdated tool timeout to: {:?}",
        server.config().tool_timeout
    );

    server.register_tool("echo", ConfigurableTool)?;

    println!("\nStarting MCP server on http://127.0.0.1:8080");
    server.serve("127.0.0.1:8080").await?;

    Ok(())
}
