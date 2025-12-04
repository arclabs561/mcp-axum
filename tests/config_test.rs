//! Tests for server configuration.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use mcp_axum::{McpServer, ServerConfig, Tool};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tower::util::ServiceExt;

struct SlowTool;

#[async_trait]
impl Tool for SlowTool {
    fn description(&self) -> &str {
        "Slow tool"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        sleep(Duration::from_secs(5)).await;
        Ok(serde_json::json!({"status": "ok"}))
    }
}

#[tokio::test]
async fn test_custom_timeout() {
    // Create server with custom 2-second timeout
    let config = ServerConfig::new().with_tool_timeout(Duration::from_secs(2));

    let mut server = McpServer::with_config(config);
    server.register_tool("slow_tool", SlowTool).unwrap();

    let app = server.router();

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

    // Should timeout around 2 seconds
    assert!(
        elapsed.as_secs() >= 2 && elapsed.as_secs() < 4,
        "Timeout should occur around 2 seconds, but took {:?}",
        elapsed
    );
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["message"].as_str().unwrap().contains("timed out"));
}

#[tokio::test]
async fn test_custom_body_size_limit() {
    let config = ServerConfig::new().with_max_body_size(1024); // 1KB limit

    let server = McpServer::with_config(config);
    let app = server.router();

    // Create a payload larger than 1KB
    let large_payload = "x".repeat(2048);
    let payload = serde_json::json!({
        "name": "test",
        "arguments": {
            "data": large_payload
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

    // Should reject oversized payload (RequestBodyLimitLayer returns 413 or 400)
    // The exact status depends on tower-http version, but it should be an error
    assert!(
        response.status() == StatusCode::PAYLOAD_TOO_LARGE
            || response.status() == StatusCode::BAD_REQUEST,
        "Expected payload rejection, got status: {:?}",
        response.status()
    );
}

#[tokio::test]
async fn test_config_access() {
    let config = ServerConfig::new().with_tool_timeout(Duration::from_secs(60));

    let server = McpServer::with_config(config.clone());

    assert_eq!(server.config().tool_timeout, Duration::from_secs(60));
    assert_eq!(server.config().max_body_size, 10 * 1024 * 1024); // Default
}

#[tokio::test]
async fn test_config_mutation() {
    let mut server = McpServer::new();

    let new_timeout = Duration::from_secs(120);
    server.config_mut().tool_timeout = new_timeout;

    assert_eq!(server.config().tool_timeout, new_timeout);
}
