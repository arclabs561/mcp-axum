//! End-to-end integration test that starts a real server and tests it with HTTP requests.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use axum_mcp::{extract_string, McpServer, Tool};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::util::ServiceExt;

struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn description(&self) -> &str {
        "Echo back the input text"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "Text to echo back"
                }
            },
            "required": ["text"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let text = extract_string(arguments, "text")?;
        Ok(json!({ "echoed": text }))
    }
}

struct FailingTool;

#[async_trait]
impl Tool for FailingTool {
    fn description(&self) -> &str {
        "A tool that always fails"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        Err("This tool always fails".to_string())
    }
}

fn create_test_server() -> axum::routing::Router {
    let server = McpServer::new()
        .tool("echo", EchoTool)
        .unwrap()
        .tool("failing", FailingTool)
        .unwrap();
    server.router()
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_server();

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_list_tools() {
    let app = create_test_server();

    let request = Request::builder()
        .uri("/tools/list")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["tools"].is_array());
    assert_eq!(json["tools"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_call_tool_success() {
    let app = create_test_server();

    let request_body = json!({
        "name": "echo",
        "arguments": {
            "text": "hello world"
        }
    });

    let request = Request::builder()
        .uri("/tools/call")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    // Response structure: {"content": [{"type": "text", "text": "<serialized JSON>"}]}
    let text_content = json["content"][0]["text"].as_str().unwrap();
    let result: Value = serde_json::from_str(text_content).unwrap();
    assert_eq!(result["echoed"], "hello world");
}

#[tokio::test]
async fn test_call_tool_failure() {
    let app = create_test_server();

    let request_body = json!({
        "name": "failing",
        "arguments": {}
    });

    let request = Request::builder()
        .uri("/tools/call")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    // Error response structure: {"code": 500, "message": "...", "details": null}
    assert!(json["message"].is_string());
    assert_eq!(json["code"], 500);
}

#[tokio::test]
async fn test_call_tool_not_found() {
    let app = create_test_server();

    let request_body = json!({
        "name": "nonexistent",
        "arguments": {}
    });

    let request = Request::builder()
        .uri("/tools/call")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_call_tool_missing_parameter() {
    let app = create_test_server();

    let request_body = json!({
        "name": "echo",
        "arguments": {}
    });

    let request = Request::builder()
        .uri("/tools/call")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    // Should return 400 Bad Request for missing required parameter
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_call_tool_invalid_json() {
    let app = create_test_server();

    let request = Request::builder()
        .uri("/tools/call")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_duplicate_registration_overwrites() {
    struct Tool1;
    struct Tool2;

    #[async_trait]
    impl Tool for Tool1 {
        fn description(&self) -> &str {
            "Tool 1"
        }
        fn schema(&self) -> Value {
            json!({})
        }
        async fn call(&self, _: &Value) -> Result<Value, String> {
            Ok(json!({"tool": 1}))
        }
    }

    #[async_trait]
    impl Tool for Tool2 {
        fn description(&self) -> &str {
            "Tool 2"
        }
        fn schema(&self) -> Value {
            json!({})
        }
        async fn call(&self, _: &Value) -> Result<Value, String> {
            Ok(json!({"tool": 2}))
        }
    }

    let mut server = McpServer::new();
    server.register_tool("test", Tool1).unwrap();
    server.register_tool("test", Tool2).unwrap(); // Should overwrite

    let app = server.router();
    let request_body = json!({
        "name": "test",
        "arguments": {}
    });

    let request = Request::builder()
        .uri("/tools/call")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    // Response structure: {"content": [{"type": "text", "text": "<serialized JSON>"}]}
    let text_content = json["content"][0]["text"].as_str().unwrap();
    let result: Value = serde_json::from_str(text_content).unwrap();
    assert_eq!(result["tool"], 2); // Should be Tool2
}
