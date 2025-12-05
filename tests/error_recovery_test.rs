//! Tests for error recovery and resilience.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use axum_mcp::{McpServer, Tool};
use http_body_util::BodyExt;
use serde_json::Value;
use std::sync::Arc;
use tower::util::ServiceExt;

struct FailingTool {
    fail_count: Arc<std::sync::atomic::AtomicUsize>,
    max_failures: usize,
}

impl FailingTool {
    fn new(max_failures: usize) -> Self {
        Self {
            fail_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            max_failures,
        }
    }
}

#[async_trait]
impl Tool for FailingTool {
    fn description(&self) -> &str {
        "A tool that fails initially then succeeds"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        let count = self
            .fail_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if count < self.max_failures {
            Err(format!("Simulated failure {}", count + 1))
        } else {
            Ok(serde_json::json!({ "status": "success" }))
        }
    }
}

#[tokio::test]
async fn test_error_recovery_after_failures() {
    let mut server = McpServer::new();
    server
        .register_tool("failing_tool", FailingTool::new(2))
        .unwrap();
    let app = server.router();

    // First call should fail
    let payload = serde_json::json!({
        "name": "failing_tool",
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

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Second call should also fail
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

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Third call should succeed
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

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["content"][0]["text"], r#"{"status":"success"}"#);
}

#[tokio::test]
async fn test_server_continues_after_errors() {
    let mut server = McpServer::new();
    server.register_tool("tool1", FailingTool::new(0)).unwrap();
    server.register_tool("tool2", FailingTool::new(0)).unwrap();
    let app = server.router();

    // Call tool1 (should succeed)
    let payload1 = serde_json::json!({
        "name": "tool1",
        "arguments": {}
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tools/call")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload1).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Call tool2 (should also succeed - server continues working)
    let payload2 = serde_json::json!({
        "name": "tool2",
        "arguments": {}
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tools/call")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload2).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
