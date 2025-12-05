//! Stress tests for high load scenarios.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use axum_mcp::{McpServer, Tool};
use serde_json::Value;
use std::time::Instant;
use tower::util::ServiceExt;

struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn description(&self) -> &str {
        "Echo tool for stress testing"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to echo"
                }
            },
            "required": ["message"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'message' parameter".to_string())?;

        Ok(serde_json::json!({ "echoed": message }))
    }
}

fn create_test_server() -> axum::routing::Router {
    let mut server = McpServer::new();
    server.register_tool("echo", EchoTool).unwrap();
    server.router()
}

#[tokio::test]
async fn test_high_concurrency() {
    let app = create_test_server();
    let start = Instant::now();
    let mut handles = Vec::new();

    // Spawn 1000 concurrent requests
    for i in 0..1000 {
        let app = app.clone();
        let handle = tokio::spawn(async move {
            let payload = serde_json::json!({
                "name": "echo",
                "arguments": {
                    "message": format!("test-{}", i)
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
            response
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.await.is_ok() {
            success_count += 1;
        }
    }

    let elapsed = start.elapsed();
    println!("1000 concurrent requests completed in {:?}", elapsed);
    println!("Success rate: {}/1000", success_count);

    // Should complete in reasonable time and have high success rate
    assert!(
        elapsed.as_secs() < 30,
        "High concurrency test took too long: {:?}",
        elapsed
    );
    assert!(
        success_count >= 950,
        "Success rate too low: {}/1000",
        success_count
    );
}

#[tokio::test]
async fn test_rapid_sequential_requests() {
    let app = create_test_server();
    let start = Instant::now();

    // Make 500 rapid sequential requests
    for i in 0..500 {
        let payload = serde_json::json!({
            "name": "echo",
            "arguments": {
                "message": format!("rapid-{}", i)
            }
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

        assert_eq!(response.status(), StatusCode::OK);
    }

    let elapsed = start.elapsed();
    println!("500 rapid sequential requests completed in {:?}", elapsed);

    // Should complete reasonably quickly
    assert!(
        elapsed.as_secs() < 60,
        "Rapid sequential requests took too long: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_mixed_operations() {
    let app = create_test_server();
    let start = Instant::now();
    let mut handles = Vec::new();

    // Mix of different operations
    for i in 0..100 {
        // Tool calls
        if i % 3 == 0 {
            let app_clone = app.clone();
            let handle = tokio::spawn(async move {
                let payload = serde_json::json!({
                    "name": "echo",
                    "arguments": { "message": format!("call-{}", i) }
                });
                app_clone
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/tools/call")
                            .header("content-type", "application/json")
                            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                            .unwrap(),
                    )
                    .await
            });
            handles.push(handle);
        }

        // List tools
        if i % 3 == 1 {
            let app_clone = app.clone();
            let handle = tokio::spawn(async move {
                app_clone
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri("/tools/list")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
            });
            handles.push(handle);
        }

        // Health check
        if i % 3 == 2 {
            let app_clone = app.clone();
            let handle = tokio::spawn(async move {
                app_clone
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri("/health")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
            });
            handles.push(handle);
        }
    }

    // Wait for all operations
    for handle in handles {
        let _ = handle.await;
    }

    let elapsed = start.elapsed();
    println!("100 mixed operations completed in {:?}", elapsed);

    assert!(
        elapsed.as_secs() < 10,
        "Mixed operations took too long: {:?}",
        elapsed
    );
}
