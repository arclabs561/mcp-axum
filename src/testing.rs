//! Testing utilities for MCP servers.
//!
//! This module provides helpers to make testing MCP tools, resources, and prompts easier.
//!
//! # Example
//!
//! ```rust,no_run
//! use mcp_axum::{test_tool, Tool};
//! use async_trait::async_trait;
//! use serde_json::{json, Value};
//!
//! struct MyTool;
//!
//! #[async_trait]
//! impl Tool for MyTool {
//!     fn description(&self) -> &str { "Test tool" }
//!     fn schema(&self) -> Value { json!({}) }
//!     async fn call(&self, _args: &Value) -> Result<Value, String> {
//!         Ok(json!({"echoed": "hello"}))
//!     }
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let tool = MyTool;
//! let args = json!({"text": "hello"});
//! let result = test_tool(&tool, args).await?;
//! assert_eq!(result["echoed"], "hello");
//! # Ok(())
//! # }
//! ```

use crate::tool::Tool;
use serde_json::Value;

/// Test a tool with given arguments.
///
/// This is a convenience function that calls `tool.call()` with the provided arguments
/// and returns the result. Useful for unit testing tools without starting a full server.
///
/// # Example
///
/// ```rust,no_run
/// use mcp_axum::{test_tool, Tool};
/// use async_trait::async_trait;
/// use serde_json::{json, Value};
///
/// struct MyTool;
///
/// #[async_trait]
/// impl Tool for MyTool {
///     fn description(&self) -> &str { "Test tool" }
///     fn schema(&self) -> Value { json!({}) }
///     async fn call(&self, _args: &Value) -> Result<Value, String> {
///         Ok(json!({"echoed": "hello"}))
///     }
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tool = MyTool;
/// let args = json!({"text": "hello"});
/// let result = test_tool(&tool, args).await?;
/// assert_eq!(result["echoed"], "hello");
/// # Ok(())
/// # }
/// ```
pub async fn test_tool(tool: &dyn Tool, arguments: Value) -> Result<Value, String> {
    tool.call(&arguments).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tool;
    use async_trait::async_trait;
    use serde_json::json;

    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn description(&self) -> &str {
            "Test tool"
        }

        fn schema(&self) -> Value {
            json!({
                "type": "object",
                "properties": {
                    "value": {"type": "string"}
                },
                "required": ["value"]
            })
        }

        async fn call(&self, arguments: &Value) -> Result<Value, String> {
            let value = arguments
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing 'value'".to_string())?;
            Ok(json!({ "result": value }))
        }
    }

    #[tokio::test]
    async fn test_test_tool() {
        let tool = TestTool;
        let args = json!({"value": "test"});
        let result = test_tool(&tool, args).await.unwrap();
        assert_eq!(result["result"], "test");
    }
}
