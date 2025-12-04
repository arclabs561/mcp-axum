//! Example showing how to add authentication middleware to an MCP server.
//!
//! This demonstrates:
//! - API key authentication
//! - Custom middleware
//! - Protected endpoints

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use mcp_axum::{extract_string, McpServer, Tool};
use serde_json::Value;
use std::{collections::HashSet, env};

use async_trait::async_trait;

/// Example authenticated tool
struct ProtectedTool;

#[async_trait]
impl Tool for ProtectedTool {
    fn description(&self) -> &str {
        "A protected tool that requires authentication"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "string",
                    "description": "Data to process"
                }
            },
            "required": ["data"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let data = extract_string(arguments, "data")?;
        Ok(serde_json::json!({
            "processed": format!("Protected: {}", data)
        }))
    }
}

/// Authentication middleware
async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let headers = request.headers();
    // Get API key from environment or use default for demo
    let valid_keys: HashSet<String> = env::var("MCP_API_KEYS")
        .unwrap_or_else(|_| "demo-key-123,test-key-456".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Check for API key in Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Support both "Bearer <key>" and just "<key>" formats
    let api_key = auth_header.strip_prefix("Bearer ").unwrap_or(auth_header);

    if !valid_keys.contains(api_key) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Add API key to request extensions for use in handlers
    // (In a real app, you might extract user info, permissions, etc.)
    Ok(next.run(request).await)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Create server
    let mut server = McpServer::new();
    server.register_tool("protected", ProtectedTool)?;

    // Get the router and add auth middleware
    let app = server
        .router()
        .layer(axum::middleware::from_fn(auth_middleware));

    let addr = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("Server running on http://{}", addr);
    tracing::info!("API keys (set MCP_API_KEYS env var to customize):");
    tracing::info!("  - demo-key-123");
    tracing::info!("  - test-key-456");
    tracing::info!("\nTest with:");
    tracing::info!(
        "  curl -H 'Authorization: Bearer demo-key-123' http://localhost:8080/tools/list"
    );

    axum::serve(listener, app).await?;
    Ok(())
}
