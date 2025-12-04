//! Performance and load tests.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use mcp_axum::{McpServer, Tool};
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tower::util::ServiceExt;

struct FastTool;

#[async_trait]
impl Tool for FastTool {
    fn description(&self) -> &str {
        "Fast tool for performance testing"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "iterations": {
                    "type": "integer",
                    "default": 1,
                    "description": "Number of iterations"
                }
            },
            "required": []
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let iterations = arguments
            .get("iterations")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        let mut sum = 0u64;
        for _ in 0..iterations {
            sum += 1;
        }

        Ok(serde_json::json!({ "sum": sum }))
    }
}

fn create_test_server() -> axum::routing::Router {
    let mut server = McpServer::new();
    server.register_tool("fast_tool", FastTool).unwrap();
    server.router()
}

#[tokio::test]
async fn test_concurrent_requests() {
    let app = create_test_server();
    let mut handles = Vec::new();

    let start = Instant::now();

    // Spawn 100 concurrent requests
    for i in 0..100 {
        let app = app.clone();
        let handle = tokio::spawn(async move {
            let payload = serde_json::json!({
                "name": "fast_tool",
                "arguments": { "iterations": i }
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
    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed = start.elapsed();
    println!("100 concurrent requests completed in {:?}", elapsed);

    // Should complete reasonably quickly (under 5 seconds for 100 requests)
    assert!(
        elapsed.as_secs() < 5,
        "Concurrent requests took too long: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_sequential_requests() {
    let app = create_test_server();
    let start = Instant::now();

    // Make 50 sequential requests
    for i in 0..50 {
        let payload = serde_json::json!({
            "name": "fast_tool",
            "arguments": { "iterations": i }
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
    println!("50 sequential requests completed in {:?}", elapsed);

    // Sequential requests should still be reasonably fast
    assert!(
        elapsed.as_secs() < 10,
        "Sequential requests took too long: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_list_tools_performance() {
    let app = create_test_server();
    let start = Instant::now();

    // Make 100 requests to list tools
    for _ in 0..100 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/tools/list")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    let elapsed = start.elapsed();
    println!("100 list_tools requests completed in {:?}", elapsed);

    // Should be very fast (under 2 seconds)
    assert!(
        elapsed.as_secs() < 2,
        "List tools requests took too long: {:?}",
        elapsed
    );
}
