//! Tests for timeout handling.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use axum_mcp::{McpServer, Resource, Tool};
use http_body_util::BodyExt;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use tower::util::ServiceExt;

struct SlowTool;

#[async_trait]
impl Tool for SlowTool {
    fn description(&self) -> &str {
        "A tool that takes a long time to execute"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        // Sleep for 35 seconds (longer than the 30 second timeout)
        sleep(Duration::from_secs(35)).await;
        Ok(serde_json::json!({"status": "completed"}))
    }
}

struct FastTool;

#[async_trait]
impl Tool for FastTool {
    fn description(&self) -> &str {
        "A tool that executes quickly"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        sleep(Duration::from_millis(100)).await;
        Ok(serde_json::json!({"status": "completed"}))
    }
}

struct SlowResource;

#[async_trait]
impl Resource for SlowResource {
    fn name(&self) -> &str {
        "slow_resource"
    }

    fn description(&self) -> &str {
        "A resource that takes a long time to read"
    }

    fn mime_type(&self) -> &str {
        "text/plain"
    }

    async fn read(&self) -> Result<String, String> {
        // Sleep for 35 seconds (longer than the 30 second timeout)
        sleep(Duration::from_secs(35)).await;
        Ok("content".to_string())
    }
}

fn create_test_server() -> axum::routing::Router {
    let mut server = McpServer::new();
    server.register_tool("slow_tool", SlowTool).unwrap();
    server.register_tool("fast_tool", FastTool).unwrap();
    server
        .register_resource("slow://resource", SlowResource)
        .unwrap();
    server.router()
}

#[tokio::test]
async fn test_tool_timeout() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "slow_tool",
        "arguments": {}
    });

    let start = std::time::Instant::now();
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

    let elapsed = start.elapsed();

    // Should timeout around 30 seconds (allow some margin for test overhead)
    assert!(
        elapsed.as_secs() >= 30 && elapsed.as_secs() < 40,
        "Timeout should occur around 30 seconds, but took {:?}",
        elapsed
    );
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["message"].as_str().unwrap().contains("timed out"));
}

#[tokio::test]
async fn test_tool_no_timeout() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "fast_tool",
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

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["content"][0]["text"], r#"{"status":"completed"}"#);
}

#[tokio::test]
async fn test_resource_timeout() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "uri": "slow://resource"
    });

    let start = std::time::Instant::now();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/resources/read")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let elapsed = start.elapsed();

    // Should timeout around 30 seconds (allow some margin for test overhead)
    assert!(
        elapsed.as_secs() >= 30 && elapsed.as_secs() < 40,
        "Timeout should occur around 30 seconds, but took {:?}",
        elapsed
    );
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["message"].as_str().unwrap().contains("timed out"));
}
