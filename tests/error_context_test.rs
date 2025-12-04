//! Tests for error context and detailed error messages.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use mcp_axum::{McpServer, Tool};
use serde_json::Value;
use tower::util::ServiceExt;

struct FailingTool {
    error_message: String,
}

#[async_trait]
impl Tool for FailingTool {
    fn description(&self) -> &str {
        "A tool that always fails"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "param": {
                    "type": "string",
                    "description": "A parameter"
                }
            },
            "required": ["param"]
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        Err(self.error_message.clone())
    }
}

fn create_test_server() -> axum::routing::Router {
    let mut server = McpServer::new();
    server
        .register_tool(
            "failing_tool",
            FailingTool {
                error_message: "Custom error message".to_string(),
            },
        )
        .unwrap();
    server.router()
}

#[tokio::test]
async fn test_tool_error_context() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "failing_tool",
        "arguments": {
            "param": "value"
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tools/call")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Error message should include the original error
    let message = json["message"].as_str().unwrap();
    // The error message format is "Tool execution failed: {error}"
    assert!(message.contains("Tool execution failed"));
    assert!(message.contains("Custom error message"));
}

#[tokio::test]
async fn test_validation_error_context() {
    let app = create_test_server();

    // Missing required parameter
    let payload = serde_json::json!({
        "name": "failing_tool",
        "arguments": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tools/call")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Error message should include tool name and validation details
    let message = json["message"].as_str().unwrap();
    assert!(message.contains("failing_tool"));
    assert!(message.contains("schema validation"));
}

#[tokio::test]
async fn test_not_found_error_context() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "nonexistent_tool",
        "arguments": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tools/call")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Error message should include the tool name that wasn't found
    let message = json["message"].as_str().unwrap();
    assert!(message.contains("nonexistent_tool"));
    assert!(message.contains("not found"));
}
