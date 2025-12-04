//! Example demonstrating utility functions for argument extraction.
//!
//! This shows how to use the helper functions to reduce boilerplate
//! when extracting arguments from tool calls.

use async_trait::async_trait;
use mcp_axum::{extract_integer_opt, extract_string, McpServer, Tool};
use serde_json::Value;

/// Example tool using utility functions for cleaner argument extraction.
struct SearchTool;

#[async_trait]
impl Tool for SearchTool {
    fn description(&self) -> &str {
        "Search for items with optional limit"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum results",
                    "default": 10
                }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        // Use utility functions instead of manual extraction
        let query = extract_string(arguments, "query")?;
        let limit = extract_integer_opt(arguments, "limit").unwrap_or(10);

        // Simulate search
        Ok(serde_json::json!({
            "query": query,
            "limit": limit,
            "results": vec!["result1", "result2", "result3"]
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let mut server = McpServer::new();
    server.register_tool("search", SearchTool)?;

    println!("Server running on http://127.0.0.1:8080");
    println!("Try: curl -X POST http://localhost:8080/tools/call -H 'Content-Type: application/json' -d '{{\"name\":\"search\",\"arguments\":{{\"query\":\"test\",\"limit\":5}}}}'");

    server.serve("127.0.0.1:8080").await?;
    Ok(())
}
