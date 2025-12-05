//! Integration tests for MCP server HTTP endpoints.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use axum_mcp::{McpServer, Prompt, Resource, Tool};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::util::ServiceExt;

struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn description(&self) -> &str {
        "Echo back the input text"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
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
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'text' parameter".to_string())?;
        Ok(serde_json::json!({ "echoed": text }))
    }
}

struct FailingTool;

#[async_trait]
impl Tool for FailingTool {
    fn description(&self) -> &str {
        "A tool that always fails"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        Err("Tool execution failed".to_string())
    }
}

struct TestResource;

#[async_trait]
impl Resource for TestResource {
    fn name(&self) -> &str {
        "Test Resource"
    }

    fn description(&self) -> &str {
        "A test resource"
    }

    fn mime_type(&self) -> &str {
        "text/plain"
    }

    async fn read(&self) -> Result<String, String> {
        Ok("test content".to_string())
    }
}

struct FailingResource;

#[async_trait]
impl Resource for FailingResource {
    fn name(&self) -> &str {
        "Failing Resource"
    }

    fn description(&self) -> &str {
        "A resource that always fails"
    }

    fn mime_type(&self) -> &str {
        "text/plain"
    }

    async fn read(&self) -> Result<String, String> {
        Err("Resource read failed".to_string())
    }
}

struct TestPrompt;

#[async_trait]
impl Prompt for TestPrompt {
    fn description(&self) -> &str {
        "A test prompt"
    }

    fn arguments(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name to greet",
                    "default": "World"
                }
            }
        })
    }

    async fn render(&self, arguments: &Value) -> Result<String, String> {
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");
        Ok(format!("Hello, {}!", name))
    }
}

struct FailingPrompt;

#[async_trait]
impl Prompt for FailingPrompt {
    fn description(&self) -> &str {
        "A prompt that always fails"
    }

    fn arguments(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn render(&self, _arguments: &Value) -> Result<String, String> {
        Err("Prompt render failed".to_string())
    }
}

fn create_test_server() -> axum::Router {
    let mut server = McpServer::new();
    server.register_tool("echo", EchoTool).unwrap();
    server.register_tool("failing", FailingTool).unwrap();
    server
        .register_resource("test://resource", TestResource)
        .unwrap();
    server
        .register_resource("test://failing", FailingResource)
        .unwrap();
    server.register_prompt("greeting", TestPrompt).unwrap();
    server.register_prompt("failing", FailingPrompt).unwrap();
    server.router()
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_server();

    let response = app
        .oneshot(
            Request::builder()
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
}

#[tokio::test]
async fn test_list_tools() {
    let app = create_test_server();

    let response = app
        .oneshot(
            Request::builder()
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
    let tools = json["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);

    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"echo"));
    assert!(tool_names.contains(&"failing"));
}

#[tokio::test]
async fn test_call_tool_success() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "echo",
        "arguments": {
            "text": "hello"
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
    let content = &json["content"][0];
    assert_eq!(content["type"], "text");
    let text: Value = serde_json::from_str(content["text"].as_str().unwrap()).unwrap();
    assert_eq!(text["echoed"], "hello");
}

#[tokio::test]
async fn test_call_tool_missing_name() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "arguments": {
            "text": "hello"
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
    assert!(json["message"].as_str().unwrap().contains("Missing 'name'"));
}

#[tokio::test]
async fn test_call_tool_not_found() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "nonexistent",
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
    assert!(json
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("not found"));
}

#[tokio::test]
async fn test_call_tool_failure() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "failing",
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

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["message"].as_str().unwrap().contains("failed"));
}

#[tokio::test]
async fn test_list_resources() {
    let app = create_test_server();

    let response = app
        .oneshot(
            Request::builder()
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
    let resources = json["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 2);
}

#[tokio::test]
async fn test_read_resource_success() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "uri": "test://resource"
    });

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

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["contents"].is_array());
    let content = &json["contents"][0];
    assert_eq!(content["uri"], "test://resource");
    assert_eq!(content["text"], "test content");
}

#[tokio::test]
async fn test_read_resource_missing_uri() {
    let app = create_test_server();

    let payload = serde_json::json!({});

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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_read_resource_not_found() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "uri": "test://nonexistent"
    });

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

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_read_resource_failure() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "uri": "test://failing"
    });

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

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_list_prompts() {
    let app = create_test_server();

    let response = app
        .oneshot(
            Request::builder()
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
    let prompts = json["prompts"].as_array().unwrap();
    assert_eq!(prompts.len(), 2);
}

#[tokio::test]
async fn test_get_prompt_success() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "greeting",
        "arguments": {
            "name": "Alice"
        }
    });

    let response = app
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
    let message = &json["messages"][0];
    assert_eq!(message["role"], "user");
    assert_eq!(message["content"]["text"], "Hello, Alice!");
}

#[tokio::test]
async fn test_get_prompt_with_default() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "greeting",
        "arguments": {}
    });

    let response = app
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
    let message = &json["messages"][0];
    assert_eq!(message["content"]["text"], "Hello, World!");
}

#[tokio::test]
async fn test_get_prompt_missing_name() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "arguments": {}
    });

    let response = app
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_prompt_not_found() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "nonexistent",
        "arguments": {}
    });

    let response = app
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

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_prompt_failure() {
    let app = create_test_server();

    let payload = serde_json::json!({
        "name": "failing",
        "arguments": {}
    });

    let response = app
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

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
