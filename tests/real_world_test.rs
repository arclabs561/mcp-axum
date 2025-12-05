//! Real-world usage scenario tests.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use axum_mcp::{McpServer, Prompt, Resource, Tool};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::util::ServiceExt;

// Simulate a real-world search tool
struct SearchTool {
    max_results: usize,
}

impl SearchTool {
    fn new(max_results: usize) -> Self {
        Self { max_results }
    }
}

#[async_trait]
impl Tool for SearchTool {
    fn description(&self) -> &str {
        "Search for documents matching a query"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results",
                    "default": 10,
                    "minimum": 1,
                    "maximum": 100
                }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'query' parameter".to_string())?;

        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let actual_limit = limit.min(self.max_results);

        // Simulate search results
        let results: Vec<Value> = (0..actual_limit)
            .map(|i| {
                json!({
                    "id": i + 1,
                    "title": format!("Result {} for '{}'", i + 1, query),
                    "snippet": format!("This is a snippet for result {}", i + 1),
                    "score": 1.0 - (i as f64 * 0.1)
                })
            })
            .collect();

        Ok(json!({
            "query": query,
            "results": results,
            "total": results.len()
        }))
    }
}

// Simulate a real-world file resource
struct FileResource {
    content: String,
    mime_type: String,
}

impl FileResource {
    fn new(content: String, mime_type: String) -> Self {
        Self { content, mime_type }
    }
}

#[async_trait]
impl Resource for FileResource {
    fn name(&self) -> &str {
        "file"
    }

    fn description(&self) -> &str {
        "File content resource"
    }

    fn mime_type(&self) -> &str {
        &self.mime_type
    }

    async fn read(&self) -> Result<String, String> {
        Ok(self.content.clone())
    }
}

// Simulate a real-world code review prompt
struct CodeReviewPrompt;

#[async_trait]
impl Prompt for CodeReviewPrompt {
    fn description(&self) -> &str {
        "Generate a code review prompt for the given code"
    }

    fn arguments(&self) -> Value {
        json!([
            {
                "name": "code",
                "description": "The code to review",
                "required": true
            },
            {
                "name": "focus",
                "description": "What to focus on (performance, security, style)",
                "required": false
            }
        ])
    }

    async fn render(&self, arguments: &Value) -> Result<String, String> {
        let code = arguments
            .get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'code' argument".to_string())?;

        let focus = arguments
            .get("focus")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        Ok(format!(
            "Please review the following code with a focus on {}:\n\n```\n{}\n```\n\nProvide feedback on code quality, potential bugs, and best practices.",
            focus, code
        ))
    }
}

fn create_real_world_server() -> axum::routing::Router {
    let mut server = McpServer::new();
    server.register_tool("search", SearchTool::new(50)).unwrap();
    server
        .register_resource(
            "file:///document.txt",
            FileResource::new(
                "This is document content".to_string(),
                "text/plain".to_string(),
            ),
        )
        .unwrap();
    server
        .register_prompt("code_review", CodeReviewPrompt)
        .unwrap();
    server.router()
}

#[tokio::test]
async fn test_search_workflow() {
    let app = create_real_world_server();

    // Search with default limit
    let payload = json!({
        "name": "search",
        "arguments": {
            "query": "rust async"
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
    assert!(json["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("rust async"));
    assert!(json["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("results"));
}

#[tokio::test]
async fn test_search_with_custom_limit() {
    let app = create_real_world_server();

    let payload = json!({
        "name": "search",
        "arguments": {
            "query": "test",
            "limit": 5
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
    let content: Value =
        serde_json::from_str(json["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(content["results"].as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn test_file_resource_workflow() {
    let app = create_real_world_server();

    // List resources
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
    assert!(!json["resources"].as_array().unwrap().is_empty());

    // Read resource
    let payload = json!({
        "uri": "file:///document.txt"
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
    assert!(json["contents"][0]["text"]
        .as_str()
        .unwrap()
        .contains("document content"));
}

#[tokio::test]
async fn test_code_review_prompt_workflow() {
    let app = create_real_world_server();

    // List prompts
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
    assert!(!json["prompts"].as_array().unwrap().is_empty());

    // Get prompt
    let payload = json!({
        "name": "code_review",
        "arguments": {
            "code": "fn main() { println!(\"hello\"); }",
            "focus": "security"
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
    let text = json["messages"][0]["content"]["text"].as_str().unwrap();
    assert!(text.contains("fn main()"));
    assert!(text.contains("security"));
}

#[tokio::test]
async fn test_complete_workflow() {
    let app = create_real_world_server();

    // 1. Check health
    let response = app
        .clone()
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

    // 2. List tools
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

    // 3. Use a tool
    let payload = json!({
        "name": "search",
        "arguments": { "query": "test" }
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

    // 4. Read a resource
    let payload = json!({ "uri": "file:///document.txt" });
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

    // 5. Get a prompt
    let payload = json!({
        "name": "code_review",
        "arguments": { "code": "test" }
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
}
