//! # axum-mcp
//!
//! MCP server framework built on axum with HTTP transport.
//!
//! Framework for building MCP servers in Rust using `axum`.
//! Trait-based handlers for tools, resources, and prompts. Arguments use `serde_json::Value`.
//!
//! # Model Context Protocol (MCP)
//!
//! MCP is a protocol for AI agents to interact with external tools and data sources.
//! It enables:
//!
//! - **Tools**: Functions that agents can call (e.g., "search_arxiv", "read_file")
//! - **Resources**: Data sources agents can access (e.g., "file://...", "arxiv://...")
//! - **Prompts**: Template prompts for common tasks
//!
//! # Example
//!
//! ```rust,no_run
//! use axum_mcp::{extract_string, McpServer, Tool};
//! use async_trait::async_trait;
//! use serde_json::Value;
//!
//! struct EchoTool;
//!
//! #[async_trait]
//! impl Tool for EchoTool {
//!     fn description(&self) -> &str {
//!         "Echo back the input text"
//!     }
//!
//!     fn schema(&self) -> Value {
//!         serde_json::json!({
//!             "type": "object",
//!             "properties": {
//!                 "text": {
//!                     "type": "string",
//!                     "description": "Text to echo back"
//!                 }
//!             },
//!             "required": ["text"]
//!         })
//!     }
//!
//!     async fn call(&self, arguments: &Value) -> Result<Value, String> {
//!         let text = extract_string(arguments, "text")?;
//!         Ok(serde_json::json!({ "echoed": text }))
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     tracing_subscriber::fmt::init();
//!     let mut server = McpServer::new();
//!     server.register_tool("echo", EchoTool)?;
//!     server.serve("127.0.0.1:8080").await?;
//!     Ok(())
//! }
//! ```
//!
//! # Features
//!
//! - HTTP transport with REST endpoints
//! - Trait-based implementation for tools, resources, and prompts
//! - JSON Schema validation of tool arguments
//! - Error handling with HTTP status codes

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod config;
pub mod error;
pub mod prompt;
pub mod resource;
/// Schema utilities for extracting JSON Schema from docstrings.
///
/// The `schema` module provides `extract_schema_from_docstring()` which can be used
/// in your tool's `schema()` method to generate JSON Schema from Rust docstrings.
///
/// Example:
/// ```rust,no_run
/// use axum_mcp::{Tool, schema::extract_schema_from_docstring};
/// use serde_json::Value;
///
/// struct MyTool;
///
/// impl Tool for MyTool {
///     fn schema(&self) -> Value {
///         extract_schema_from_docstring(r#"
///             # Arguments
///             * `text` - Input text (type: string)
///         "#)
///     }
///     // ...
/// }
/// ```
pub mod schema;
pub mod server;
#[cfg(feature = "testing")]
pub mod testing;
pub mod tool;
pub mod tool_error;
pub mod utils;
pub mod validation;

// Procedural macros will be in a separate axum-mcp-macros crate
// #[cfg(feature = "macros")]
// pub use axum_mcp_macros::{mcp_tool, mcp_resource, mcp_prompt};

pub use config::ServerConfig;
pub use error::{ErrorResponse, HttpError, McpError};
pub use prompt::Prompt;
pub use resource::Resource;
pub use server::McpServer;
#[cfg(feature = "testing")]
pub use testing::test_tool;
pub use tool::Tool;
pub use tool_error::{ToolError, ToolErrorResponse};
pub use utils::{
    extract_bool, extract_bool_opt, extract_integer, extract_integer_opt, extract_number,
    extract_number_opt, extract_string, extract_string_opt,
};
pub use validation::{validate_prompt_name, validate_resource_uri, validate_tool_name};
