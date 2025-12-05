//! Basic unit tests for MCP server components.

use async_trait::async_trait;
use axum_mcp::{McpServer, Prompt, Resource, Tool};
use serde_json::Value;

struct TestTool;

#[async_trait]
impl Tool for TestTool {
    fn description(&self) -> &str {
        "A test tool"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Input string"
                }
            },
            "required": ["input"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let input = arguments
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'input' parameter".to_string())?;
        Ok(serde_json::json!({ "output": input }))
    }
}

struct TestResource;

#[async_trait]
impl Resource for TestResource {
    fn name(&self) -> &str {
        "Test Resource"
    }

    fn description(&self) -> &str {
        "A test resource"
    }

    fn mime_type(&self) -> &str {
        "text/plain"
    }

    async fn read(&self) -> Result<String, String> {
        Ok("test content".to_string())
    }
}

struct TestPrompt;

#[async_trait]
impl Prompt for TestPrompt {
    fn description(&self) -> &str {
        "A test prompt"
    }

    fn arguments(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name",
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

#[tokio::test]
async fn test_server_creation() {
    let server = McpServer::new();
    let _router = server.router();
    // Router should be created successfully (no panic)
}

#[tokio::test]
async fn test_tool_registration() {
    let mut server = McpServer::new();
    server.register_tool("test", TestTool).unwrap();
    // Registration should not panic
}

#[tokio::test]
async fn test_resource_registration() {
    let mut server = McpServer::new();
    server
        .register_resource("test://resource", TestResource)
        .unwrap();
    // Registration should not panic
}

#[tokio::test]
async fn test_prompt_registration() {
    let mut server = McpServer::new();
    server.register_prompt("test", TestPrompt).unwrap();
    // Registration should not panic
}

#[tokio::test]
async fn test_tool_call() {
    let tool = TestTool;
    let args = serde_json::json!({ "input": "test" });
    let result = tool.call(&args).await;
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.get("output").and_then(|v| v.as_str()), Some("test"));
}

#[tokio::test]
async fn test_resource_read() {
    let resource = TestResource;
    let result = resource.read().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test content");
}

#[tokio::test]
async fn test_prompt_render() {
    let prompt = TestPrompt;
    let args = serde_json::json!({ "name": "Alice" });
    let result = prompt.render(&args).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, Alice!");
}
