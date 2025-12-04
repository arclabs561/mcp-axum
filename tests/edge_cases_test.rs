//! Edge case and error handling tests.

use async_trait::async_trait;
use mcp_axum::{McpServer, Prompt, Resource, Tool};
use serde_json::Value;
use std::sync::Arc;

struct EmptyTool;

#[async_trait]
impl Tool for EmptyTool {
    fn description(&self) -> &str {
        ""
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        Ok(serde_json::json!({}))
    }
}

struct LargeOutputTool;

#[async_trait]
impl Tool for LargeOutputTool {
    fn description(&self) -> &str {
        "Tool that returns large output"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        let large_vec: Vec<Value> = (0..1000)
            .map(|i| serde_json::json!({ "index": i, "data": "x".repeat(100) }))
            .collect();
        Ok(serde_json::json!({ "items": large_vec }))
    }
}

struct InvalidJsonTool;

#[async_trait]
impl Tool for InvalidJsonTool {
    fn description(&self) -> &str {
        "Tool that returns invalid JSON"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        // Return a value that contains NaN or Infinity which can't be serialized
        Ok(serde_json::json!({
            "value": f64::NAN
        }))
    }
}

struct EmptyResource;

#[async_trait]
impl Resource for EmptyResource {
    fn name(&self) -> &str {
        ""
    }

    fn description(&self) -> &str {
        ""
    }

    fn mime_type(&self) -> &str {
        "application/octet-stream"
    }

    async fn read(&self) -> Result<String, String> {
        Ok(String::new())
    }
}

struct LargeResource;

#[async_trait]
impl Resource for LargeResource {
    fn name(&self) -> &str {
        "Large Resource"
    }

    fn description(&self) -> &str {
        "A resource with large content"
    }

    fn mime_type(&self) -> &str {
        "text/plain"
    }

    async fn read(&self) -> Result<String, String> {
        Ok("x".repeat(100000))
    }
}

#[tokio::test]
async fn test_empty_tool_description() {
    let tool = EmptyTool;
    assert_eq!(tool.description(), "");
    let schema = tool.schema();
    assert_eq!(schema["type"], "object");
}

#[tokio::test]
async fn test_empty_tool_call() {
    let tool = EmptyTool;
    let args = serde_json::json!({});
    let result = tool.call(&args).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), serde_json::json!({}));
}

#[tokio::test]
async fn test_large_output() {
    let tool = LargeOutputTool;
    let args = serde_json::json!({});
    let result = tool.call(&args).await;
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value["items"].is_array());
    assert_eq!(value["items"].as_array().unwrap().len(), 1000);
}

#[tokio::test]
async fn test_invalid_json_serialization() {
    let tool = InvalidJsonTool;
    let args = serde_json::json!({});
    let result = tool.call(&args).await;
    // The call itself succeeds, but serialization might fail
    assert!(result.is_ok());
    // NaN should serialize to null in JSON
    let value = result.unwrap();
    assert!(value["value"].is_null());
}

#[tokio::test]
async fn test_empty_resource() {
    let resource = EmptyResource;
    assert_eq!(resource.name(), "");
    assert_eq!(resource.description(), "");
    let result = resource.read().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[tokio::test]
async fn test_large_resource() {
    let resource = LargeResource;
    let result = resource.read().await;
    assert!(result.is_ok());
    let content = result.unwrap();
    assert_eq!(content.len(), 100000);
}

#[tokio::test]
async fn test_multiple_registrations() {
    let mut server = McpServer::new();
    server.register_tool("tool1", EmptyTool).unwrap();
    server.register_tool("tool2", EmptyTool).unwrap();
    server.register_tool("tool1", EmptyTool).unwrap(); // Overwrite

    let _router = server.router();
    // Should not panic
}

#[tokio::test]
async fn test_empty_server() {
    let server = McpServer::new();
    let _router = server.router();
    // Empty server should still work
}

#[tokio::test]
async fn test_tool_with_missing_required_params() {
    struct RequiredParamTool;

    #[async_trait]
    impl Tool for RequiredParamTool {
        fn description(&self) -> &str {
            "Tool requiring a parameter"
        }

        fn schema(&self) -> Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "required_param": {
                        "type": "string"
                    }
                },
                "required": ["required_param"]
            })
        }

        async fn call(&self, arguments: &Value) -> Result<Value, String> {
            arguments
                .get("required_param")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required_param".to_string())?;
            Ok(serde_json::json!({ "success": true }))
        }
    }

    let tool = RequiredParamTool;
    let args = serde_json::json!({});
    let result = tool.call(&args).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("required_param"));
}

#[tokio::test]
async fn test_tool_with_extra_params() {
    struct SimpleTool;

    #[async_trait]
    impl Tool for SimpleTool {
        fn description(&self) -> &str {
            "Simple tool"
        }

        fn schema(&self) -> Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "param": {
                        "type": "string"
                    }
                },
                "required": []
            })
        }

        async fn call(&self, arguments: &Value) -> Result<Value, String> {
            // Should ignore extra parameters
            Ok(serde_json::json!({
                "received": arguments
            }))
        }
    }

    let tool = SimpleTool;
    let args = serde_json::json!({
        "param": "value",
        "extra": "ignored"
    });
    let result = tool.call(&args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_prompt_with_empty_arguments() {
    struct EmptyPrompt;

    #[async_trait]
    impl Prompt for EmptyPrompt {
        fn description(&self) -> &str {
            "Empty prompt"
        }

        fn arguments(&self) -> Value {
            serde_json::json!({
                "type": "object",
                "properties": {}
            })
        }

        async fn render(&self, _arguments: &Value) -> Result<String, String> {
            Ok("Empty".to_string())
        }
    }

    let prompt = EmptyPrompt;
    let args = serde_json::json!({});
    let result = prompt.render(&args).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Empty");
}

#[tokio::test]
async fn test_resource_with_special_mime_types() {
    struct JsonResource;

    #[async_trait]
    impl Resource for JsonResource {
        fn name(&self) -> &str {
            "JSON Resource"
        }

        fn description(&self) -> &str {
            "A JSON resource"
        }

        fn mime_type(&self) -> &str {
            "application/json"
        }

        async fn read(&self) -> Result<String, String> {
            Ok(r#"{"key": "value"}"#.to_string())
        }
    }

    let resource = JsonResource;
    assert_eq!(resource.mime_type(), "application/json");
    let result = resource.read().await;
    assert!(result.is_ok());
    let content = result.unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["key"], "value");
}
