//! Tests for request validation and schema validation.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use mcp_axum::{McpServer, Tool};
use serde_json::Value;
use std::sync::Arc;
use tower::util::ServiceExt;

struct TestTool;

#[async_trait]
impl Tool for TestTool {
    fn description(&self) -> &str {
        "A test tool with required parameter"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "required_param": {
                    "type": "string",
                    "description": "A required parameter"
                },
                "optional_param": {
                    "type": "integer",
                    "description": "An optional parameter",
                    "default": 42
                }
            },
            "required": ["required_param"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let param = arguments
            .get("required_param")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing required_param".to_string())?;
        Ok(serde_json::json!({ "result": param }))
    }
}

fn create_test_server() -> axum::Router {
    let mut server = McpServer::new();
    server.register_tool("test_tool", TestTool).unwrap();
    server.router()
}

#[tokio::test]
async fn test_schema_validation_missing_required() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "test_tool",
        "arguments": {
            "optional_param": 100
            // Missing required_param
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("Arguments for tool 'test_tool' failed schema validation"));
}

#[tokio::test]
async fn test_schema_validation_invalid_type() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "test_tool",
        "arguments": {
            "required_param": 123, // Should be string, not number
            "optional_param": "not_a_number" // Should be integer, not string
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("Arguments for tool 'test_tool' failed schema validation"));
}

#[tokio::test]
async fn test_schema_validation_valid_request() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "test_tool",
        "arguments": {
            "required_param": "test_value",
            "optional_param": 100
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

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["content"].is_array());
}

#[tokio::test]
async fn test_schema_validation_with_defaults() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "test_tool",
        "arguments": {
            "required_param": "test_value"
            // optional_param should use default
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

    assert_eq!(response.status(), StatusCode::OK);
}
