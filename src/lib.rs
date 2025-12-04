//! # mcp-axum
//!
//! Axum-like framework for building Model Context Protocol (MCP) servers with HTTP transport.
//!
//! This crate provides an ergonomic framework for building MCP servers in Rust using `axum`.
//! It supports docstring-driven schema extraction and type-safe trait-based handlers.
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
//! use mcp_axum::{McpServer, Tool};
//! use async_trait::async_trait;
//! use serde_json::Value;
//! use std::sync::Arc;
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
//!         let text = arguments
//!             .get("text")
//!             .and_then(|v| v.as_str())
//!             .ok_or_else(|| "Missing 'text' parameter".to_string())?;
//!         Ok(serde_json::json!({ "echoed": text }))
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     tracing_subscriber::fmt::init();
//!     let mut server = McpServer::new();
//!     server.register_tool("echo".to_string(), Arc::new(EchoTool))?;
//!     server.serve("127.0.0.1:8080").await?;
//!     Ok(())
//! }
//! ```
//!
//! # Features
//!
//! - **HTTP transport**: RESTful API endpoints (unlike stdio-based alternatives)
//! - **Docstring schema extraction**: Parse parameter types, descriptions, and defaults from comments
//! - **Type-safe traits**: `Tool`, `Resource`, and `Prompt` traits with async support
//! - **Framework integration**: Built on `axum` for routing, middleware, and composability
//! - **Error handling**: Comprehensive error types with proper HTTP status codes

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod server;
pub mod tool;
pub mod resource;
pub mod prompt;
pub mod schema;
pub mod error;
pub mod validation;

// Procedural macros will be in a separate mcp-axum-macros crate
// #[cfg(feature = "macros")]
// pub use mcp_axum_macros::{mcp_tool, mcp_resource, mcp_prompt};

pub use server::McpServer;
pub use tool::Tool;
pub use resource::Resource;
pub use prompt::Prompt;
pub use error::{McpError, HttpError, ErrorResponse};
pub use validation::{validate_tool_name, validate_resource_uri, validate_prompt_name};

