//! End-to-end integration test that starts a real server and tests it with HTTP requests.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use axum_mcp::{extract_string, McpServer, Prompt, Resource, Tool};
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

struct TestResource;

#[async_trait]
impl Resource for TestResource {
    fn name(&self) -> &str {
        "Test Resource"
    }

    fn description(&self) -> &str {
        "A test resource for e2e testing"
    }

    fn mime_type(&self) -> &str {
        "text/plain"
    }

    async fn read(&self) -> Result<String, String> {
        Ok("Test resource content".to_string())
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
        Err("This resource always fails".to_string())
    }
}

struct TestPrompt;

#[async_trait]
impl Prompt for TestPrompt {
    fn description(&self) -> &str {
        "A test prompt for e2e testing"
    }

    fn arguments(&self) -> Value {
        json!({
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
        let name = extract_string(arguments, "name").unwrap_or_else(|_| "World".to_string());
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
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn render(&self, _arguments: &Value) -> Result<String, String> {
        Err("This prompt always fails".to_string())
    }
}

fn create_test_server() -> axum::routing::Router {
    let server = McpServer::new()
        .tool("echo", EchoTool)
        .unwrap()
        .tool("failing", FailingTool)
        .unwrap()
        .resource("test://resource", TestResource)
        .unwrap()
        .resource("test://failing", FailingResource)
        .unwrap()
        .prompt("test_prompt", TestPrompt)
        .unwrap()
        .prompt("failing_prompt", FailingPrompt)
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

#[tokio::test]
async fn test_list_resources() {
    let app = create_test_server();

    let request = Request::builder()
        .uri("/resources/list")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["resources"].is_array());
    assert_eq!(json["resources"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_read_resource_success() {
    let app = create_test_server();

    let request_body = json!({
        "uri": "test://resource"
    });

    let request = Request::builder()
        .uri("/resources/read")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["contents"][0]["text"], "Test resource content");
}

#[tokio::test]
async fn test_read_resource_not_found() {
    let app = create_test_server();

    let request_body = json!({
        "uri": "test://nonexistent"
    });

    let request = Request::builder()
        .uri("/resources/read")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_read_resource_failure() {
    let app = create_test_server();

    let request_body = json!({
        "uri": "test://failing"
    });

    let request = Request::builder()
        .uri("/resources/read")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_list_prompts() {
    let app = create_test_server();

    let request = Request::builder()
        .uri("/prompts/list")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["prompts"].is_array());
    assert_eq!(json["prompts"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_prompt_success() {
    let app = create_test_server();

    let request_body = json!({
        "name": "test_prompt",
        "arguments": {
            "name": "Rust"
        }
    });

    let request = Request::builder()
        .uri("/prompts/get")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    // Response structure: {"messages": [{"role": "user", "content": {"type": "text", "text": "..."}}]}
    assert_eq!(json["messages"][0]["content"]["text"], "Hello, Rust!");
}

#[tokio::test]
async fn test_get_prompt_not_found() {
    let app = create_test_server();

    let request_body = json!({
        "name": "nonexistent",
        "arguments": {}
    });

    let request = Request::builder()
        .uri("/prompts/get")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_prompt_failure() {
    let app = create_test_server();

    let request_body = json!({
        "name": "failing_prompt",
        "arguments": {}
    });

    let request = Request::builder()
        .uri("/prompts/get")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
