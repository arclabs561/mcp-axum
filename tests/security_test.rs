//! Security tests for input validation and sanitization.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use axum_mcp::{McpServer, Tool};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::util::ServiceExt;

struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn description(&self) -> &str {
        "Echo tool for testing"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "Text to echo"
                }
            },
            "required": ["text"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'text' parameter".to_string())?;

        Ok(serde_json::json!({ "echoed": text }))
    }
}

fn create_test_server() -> axum::routing::Router {
    let mut server = McpServer::new();
    server.register_tool("echo", EchoTool).unwrap();
    server.router()
}

#[tokio::test]
async fn test_sql_injection_attempt() {
    let app = create_test_server();

    // Attempt SQL injection in tool name
    let payload = serde_json::json!({
        "name": "echo'; DROP TABLE users; --",
        "arguments": { "text": "test" }
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

    // Should reject invalid tool name (contains invalid characters)
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_xss_attempt() {
    let app = create_test_server();

    // Attempt XSS in arguments
    let payload = serde_json::json!({
        "name": "echo",
        "arguments": {
            "text": "<script>alert('XSS')</script>"
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

    // Should handle XSS attempt (tool receives it, but JSON encoding prevents execution)
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    // The script tag should be in the response but properly escaped in JSON
    assert!(json["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("<script>"));
}

#[tokio::test]
async fn test_path_traversal_attempt() {
    let app = create_test_server();

    // Attempt path traversal in tool name
    let payload = serde_json::json!({
        "name": "../../etc/passwd",
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

    // Should reject invalid tool name (contains invalid characters)
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_oversized_payload() {
    let app = create_test_server();

    // Create a very large payload (larger than 10MB limit)
    let large_text = "x".repeat(11 * 1024 * 1024); // 11MB
    let payload = serde_json::json!({
        "name": "echo",
        "arguments": {
            "text": large_text
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

    // Should reject oversized payload
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn test_invalid_json() {
    let app = create_test_server();

    // Send invalid JSON
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tools/call")
                .header("content-type", "application/json")
                .body(Body::from("invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should reject invalid JSON
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_missing_required_fields() {
    let app = create_test_server();

    // Missing 'name' field
    let payload = serde_json::json!({
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
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("Missing 'name' field"));
}

#[tokio::test]
async fn test_invalid_tool_name_characters() {
    let app = create_test_server();

    // Tool name with invalid characters
    let invalid_names = vec![
        "tool with spaces",
        "tool,with,commas",
        "tool@with#special$chars",
        "tool\nwith\nnewlines",
        "tool\twith\ttabs",
    ];

    for name in invalid_names {
        let payload = serde_json::json!({
            "name": name,
            "arguments": {}
        });

        let response = app
            .clone()
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
    }
}
