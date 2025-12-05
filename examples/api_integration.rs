//! Example MCP server that integrates with external APIs.
//!
//! This example demonstrates:
//! - HTTP client integration
//! - API error handling
//! - Timeout and error handling patterns
//! - JSON response parsing

use async_trait::async_trait;
use axum_mcp::{McpServer, Tool};
use serde_json::{json, Value};

/// GitHub API tool that fetches repository information.
struct GitHubTool;

#[async_trait]
impl Tool for GitHubTool {
    fn description(&self) -> &str {
        "Fetches repository information from GitHub API"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "owner": {
                    "type": "string",
                    "description": "Repository owner (username or organization)"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                }
            },
            "required": ["owner", "repo"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let owner = arguments
            .get("owner")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'owner' parameter".to_string())?;

        let repo = arguments
            .get("repo")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'repo' parameter".to_string())?;

        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

        // In a real implementation, you'd use reqwest or similar HTTP client
        // For this example, we'll return mock data
        Ok(json!({
            "owner": owner,
            "repo": repo,
            "url": url,
            "stars": 1234,
            "forks": 567,
            "description": "Example repository",
            "note": "This is a mock response. In production, use reqwest to make actual API calls."
        }))
    }
}

/// JSONPlaceholder API tool for testing.
struct JsonPlaceholderTool;

#[async_trait]
impl Tool for JsonPlaceholderTool {
    fn description(&self) -> &str {
        "Fetches posts from JSONPlaceholder API (testing API)"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "post_id": {
                    "type": "integer",
                    "description": "Post ID (1-100)",
                    "minimum": 1,
                    "maximum": 100
                }
            },
            "required": ["post_id"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let post_id = arguments
            .get("post_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "Missing or invalid 'post_id' parameter".to_string())?;

        if !(1..=100).contains(&post_id) {
            return Err("post_id must be between 1 and 100".to_string());
        }

        let url = format!("https://jsonplaceholder.typicode.com/posts/{}", post_id);

        // In a real implementation, you'd use reqwest:
        // let client = reqwest::Client::new();
        // let response = client.get(&url).send().await?;
        // let post: Value = response.json().await?;

        // For this example, return mock data
        Ok(json!({
            "id": post_id,
            "title": format!("Post Title {}", post_id),
            "body": format!("Post body content for post {}", post_id),
            "userId": 1,
            "url": url,
            "note": "This is a mock response. In production, use reqwest to make actual API calls."
        }))
    }
}

/// IP geolocation tool (mock).
struct IpGeolocationTool;

#[async_trait]
impl Tool for IpGeolocationTool {
    fn description(&self) -> &str {
        "Gets geolocation information for an IP address"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "ip": {
                    "type": "string",
                    "description": "IP address (IPv4 or IPv6)",
                    "pattern": "^([0-9]{1,3}\\.){3}[0-9]{1,3}$|^([0-9a-fA-F]{0,4}:){7}[0-9a-fA-F]{0,4}$"
                }
            },
            "required": ["ip"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let ip = arguments
            .get("ip")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'ip' parameter".to_string())?;

        // In a real implementation, you'd call an IP geolocation API
        // For this example, return mock data
        Ok(json!({
            "ip": ip,
            "country": "United States",
            "region": "California",
            "city": "San Francisco",
            "latitude": 37.7749,
            "longitude": -122.4194,
            "timezone": "America/Los_Angeles",
            "note": "This is a mock response. In production, use a real IP geolocation API."
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let mut server = McpServer::new();

    // Register API integration tools
    server.register_tool("github_repo", GitHubTool)?;
    server.register_tool("json_placeholder", JsonPlaceholderTool)?;
    server.register_tool("ip_geolocation", IpGeolocationTool)?;

    println!("Starting API integration MCP server on http://127.0.0.1:8080");
    println!("Available tools:");
    println!("  - github_repo: Fetch GitHub repository information");
    println!("  - json_placeholder: Fetch posts from JSONPlaceholder API");
    println!("  - ip_geolocation: Get geolocation for an IP address");
    println!("\nNote: These tools return mock data. In production, integrate with real APIs using reqwest or similar.");

    server.serve("127.0.0.1:8080").await?;

    Ok(())
}
