//! Example MCP server with graceful shutdown.
//!
//! This example demonstrates:
//! - Graceful shutdown on SIGINT/SIGTERM
//! - Custom configuration
//! - Proper cleanup

use async_trait::async_trait;
use axum_mcp::{McpServer, ServerConfig, Tool};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::signal;

struct HelloTool;

#[async_trait]
impl Tool for HelloTool {
    fn description(&self) -> &str {
        "Says hello"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name to greet"
                }
            },
            "required": ["name"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'name' parameter".to_string())?;

        Ok(json!({ "message": format!("Hello, {}!", name) }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Create custom configuration
    let config = ServerConfig::new()
        .with_tool_timeout(Duration::from_secs(60))
        .with_resource_timeout(Duration::from_secs(60))
        .with_max_body_size(20 * 1024 * 1024); // 20MB

    let mut server = McpServer::with_config(config);
    server.register_tool("hello", HelloTool)?;

    println!("Starting MCP server on http://127.0.0.1:8080");
    println!("Press CTRL+C to gracefully shut down");

    // Setup graceful shutdown
    let shutdown = async {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        println!("\nShutdown signal received, shutting down gracefully...");
    };

    server
        .serve_with_shutdown("127.0.0.1:8080", shutdown)
        .await?;

    println!("Server shut down successfully");
    Ok(())
}
