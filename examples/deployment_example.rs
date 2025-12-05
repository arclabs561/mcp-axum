//! Example showing how to deploy an MCP server in production.
//!
//! This demonstrates:
//! - Environment-based configuration
//! - Production-ready server setup
//! - Error handling and logging
//! - Graceful shutdown

use async_trait::async_trait;
use axum_mcp::{McpServer, ServerConfig, Tool};
use serde_json::Value;
use std::{env, time::Duration};

/// Example production tool that uses environment variables
struct DatabaseQueryTool {
    db_url: String,
}

#[async_trait]
impl Tool for DatabaseQueryTool {
    fn description(&self) -> &str {
        "Query the company database"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "SQL query to execute"
                }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let _query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'query' parameter".to_string())?;

        // In production, you'd actually execute the query
        // For this example, we just return a mock result
        tracing::info!("Executing query on database: {}", self.db_url);
        Ok(serde_json::json!({
            "rows": [
                {"id": 1, "name": "Example"},
                {"id": 2, "name": "Result"}
            ]
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with environment-based log level
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt().with_env_filter(log_level).init();

    // Load configuration from environment
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    tracing::info!("Starting MCP server on {}", bind_address);
    tracing::info!("Database URL configured: {}", db_url);

    // Create server with production configuration
    let config = ServerConfig::new()
        .with_tool_timeout(Duration::from_secs(60))
        .with_resource_timeout(Duration::from_secs(30))
        .with_prompt_timeout(Duration::from_secs(10))
        .with_max_body_size(50 * 1024 * 1024); // 50MB

    let mut server = McpServer::with_config(config);

    // Register tools
    server.register_tool("query_database", DatabaseQueryTool { db_url })?;
    tracing::info!("Server ready to accept connections");

    // Start server with graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        tracing::info!("Received shutdown signal");
    };

    server
        .serve_with_shutdown(&bind_address, shutdown_signal)
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}
