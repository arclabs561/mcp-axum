//! Concurrent access and thread safety tests.

use mcp_axum::{McpServer, Tool};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::task;

struct CounterTool {
    count: Arc<std::sync::atomic::AtomicUsize>,
}

impl CounterTool {
    fn new() -> Self {
        Self {
            count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    fn get_count(&self) -> usize {
        self.count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait]
impl Tool for CounterTool {
    fn description(&self) -> &str {
        "Counter tool"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        let current = self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(serde_json::json!({ "count": current }))
    }
}

#[tokio::test]
async fn test_concurrent_tool_calls() {
    let tool = Arc::new(CounterTool::new());
    let mut handles = Vec::new();

    for _ in 0..100 {
        let tool_clone = Arc::clone(&tool);
        let handle = task::spawn(async move {
            tool_clone
                .call(&serde_json::json!({}))
                .await
                .unwrap()
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(tool.get_count(), 100);
}

#[tokio::test]
async fn test_concurrent_server_registration() {
    let server = Arc::new(tokio::sync::Mutex::new(McpServer::new()));
    let mut handles = Vec::new();

    for i in 0..50 {
        let server_clone = Arc::clone(&server);
        let handle = task::spawn(async move {
            let mut s = server_clone.lock().await;
            s.register_tool(
                format!("tool_{}", i),
                Arc::new(CounterTool::new()),
            ).unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let s = server.lock().await;
    let _router = s.clone().router();
    // Should have all tools registered
}

#[tokio::test]
async fn test_server_clone() {
    let mut server = McpServer::new();
    server.register_tool("tool1".to_string(), Arc::new(CounterTool::new())).unwrap();
    
    let cloned = server.clone();
    let _router1 = server.router();
    let _router2 = cloned.router();
    
    // Both should work independently
}

#[tokio::test]
async fn test_multiple_servers_independent() {
    let mut server1 = McpServer::new();
    let mut server2 = McpServer::new();
    
    server1.register_tool("tool1".to_string(), Arc::new(CounterTool::new())).unwrap();
    server2.register_tool("tool2".to_string(), Arc::new(CounterTool::new())).unwrap();
    
    let _router1 = server1.router();
    let _router2 = server2.router();
    
    // Should be independent
}

