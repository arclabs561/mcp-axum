//! Comprehensive integration tests covering full request/response cycles.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use mcp_axum::{McpServer, Prompt, Resource, ServerConfig, Tool};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tower::util::ServiceExt;

struct ComprehensiveTool;

#[async_trait]
impl Tool for ComprehensiveTool {
    fn description(&self) -> &str {
        "A comprehensive test tool"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "string_param": { "type": "string" },
                "number_param": { "type": "number" },
                "boolean_param": { "type": "boolean" },
                "array_param": { "type": "array", "items": { "type": "string" } },
                "object_param": { "type": "object" },
                "optional_param": { "type": "string" }
            },
            "required": ["string_param", "number_param", "boolean_param"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        Ok(json!({
            "received": arguments,
            "status": "success"
        }))
    }
}

struct ComprehensiveResource;

#[async_trait]
impl Resource for ComprehensiveResource {
    fn name(&self) -> &str {
        "comprehensive_resource"
    }

    fn description(&self) -> &str {
        "A comprehensive test resource"
    }

    fn mime_type(&self) -> &str {
        "application/json"
    }

    async fn read(&self) -> Result<String, String> {
        Ok(json!({
            "data": "test",
            "timestamp": "2024-01-01T00:00:00Z"
        })
        .to_string())
    }
}

struct ComprehensivePrompt;

#[async_trait]
impl Prompt for ComprehensivePrompt {
    fn description(&self) -> &str {
        "A comprehensive test prompt"
    }

    fn arguments(&self) -> Value {
        json!([
            {
                "name": "topic",
                "description": "The topic",
                "required": true
            },
            {
                "name": "style",
                "description": "The style",
                "required": false
            }
        ])
    }

    async fn render(&self, arguments: &Value) -> Result<String, String> {
        let topic = arguments
            .get("topic")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'topic' argument".to_string())?;

        let style = arguments
            .get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("formal");

        Ok(format!("Write about {} in a {} style", topic, style))
    }
}

fn create_comprehensive_server() -> axum::routing::Router {
    let mut server = McpServer::new();
    server
        .register_tool("comprehensive_tool", ComprehensiveTool)
        .unwrap();
    server
        .register_resource("test://resource", ComprehensiveResource)
        .unwrap();
    server
        .register_prompt("comprehensive_prompt", ComprehensivePrompt)
        .unwrap();
    server.router()
}

#[tokio::test]
async fn test_full_tool_lifecycle() {
    let app = create_comprehensive_server();

    // 1. List tools
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["tools"].is_array());
    assert!(!json["tools"].as_array().unwrap().is_empty());

    // 2. Call tool with all parameter types
    let payload = json!({
        "name": "comprehensive_tool",
        "arguments": {
            "string_param": "test",
            "number_param": 42,
            "boolean_param": true,
            "array_param": ["a", "b", "c"],
            "object_param": { "key": "value" },
            "optional_param": "optional"
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["content"][0]["type"], "text");
}

#[tokio::test]
async fn test_full_resource_lifecycle() {
    let app = create_comprehensive_server();

    // 1. List resources
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/resources/list")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["resources"].is_array());

    // 2. Read resource
    let payload = json!({
        "uri": "test://resource"
    });

    let response = app
        .clone()
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

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["contents"].is_array());
}

#[tokio::test]
async fn test_full_prompt_lifecycle() {
    let app = create_comprehensive_server();

    // 1. List prompts
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/prompts/list")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["prompts"].is_array());

    // 2. Get prompt
    let payload = json!({
        "name": "comprehensive_prompt",
        "arguments": {
            "topic": "Rust programming",
            "style": "technical"
        }
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/prompts/get")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["messages"].is_array());
}

#[tokio::test]
async fn test_health_endpoint_with_state() {
    let app = create_comprehensive_server();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert!(json["tools"].is_number());
    assert!(json["resources"].is_number());
    assert!(json["prompts"].is_number());
    assert!(json["version"].is_string());
}

#[tokio::test]
async fn test_configuration_affects_behavior() {
    let config = ServerConfig::new().with_tool_timeout(Duration::from_secs(1));

    let mut server = McpServer::with_config(config);
    server.register_tool("slow_tool", SlowTool).unwrap();

    let app = server.router();

    let payload = json!({
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

    // Should timeout around 1 second (not 30)
    assert!(
        elapsed.as_secs() >= 1 && elapsed.as_secs() < 3,
        "Timeout should occur around 1 second, but took {:?}",
        elapsed
    );
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

struct SlowTool;

#[async_trait]
impl Tool for SlowTool {
    fn description(&self) -> &str {
        "Slow tool"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        tokio::time::sleep(Duration::from_secs(5)).await;
        Ok(json!({"status": "ok"}))
    }
}
