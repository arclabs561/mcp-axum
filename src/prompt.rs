//! Prompt template definitions for MCP.
//!
//! Prompts are reusable template messages that help users accomplish specific tasks.
//! They accept arguments and render to formatted text that can be sent to LLMs.
//!
//! # Example
//!
//! ```rust,no_run
//! use mcp_axum::Prompt;
//! use async_trait::async_trait;
//! use serde_json::{Value, json};
//!
//! struct GreetingPrompt;
//!
//! #[async_trait]
//! impl Prompt for GreetingPrompt {
//!     fn description(&self) -> &str {
//!         "Generate a greeting message"
//!     }
//!
//!     fn arguments(&self) -> Value {
//!         json!({
//!             "type": "object",
//!             "properties": {
//!                 "name": {
//!                     "type": "string",
//!                     "description": "Name to greet",
//!                     "default": "World"
//!                 }
//!             }
//!         })
//!     }
//!
//!     async fn render(&self, arguments: &Value) -> Result<String, String> {
//!         let name = arguments
//!             .get("name")
//!             .and_then(|v| v.as_str())
//!             .unwrap_or("World");
//!         Ok(format!("Hello, {}!", name))
//!     }
//! }
//! ```

use async_trait::async_trait;
use serde_json::Value;

/// A prompt template that can be rendered by MCP clients.
///
/// Prompts are pre-written message templates that help users accomplish common tasks.
/// They accept arguments (defined via JSON Schema) and render to formatted text.
///
/// # Use Cases
///
/// - Code review prompts with project context
/// - Documentation generation templates
/// - Standardized query formats
/// - Multi-step task instructions
#[async_trait]
pub trait Prompt: Send + Sync {
    /// Get the prompt's description.
    ///
    /// Describe what this prompt does and when users should use it.
    fn description(&self) -> &str;

    /// Get the JSON Schema for the prompt's arguments.
    ///
    /// This defines what parameters the prompt accepts when rendering.
    /// Follows the same format as `Tool::schema()`.
    fn arguments(&self) -> Value;

    /// Render the prompt with the given arguments.
    ///
    /// # Arguments
    ///
    /// * `arguments` - A JSON object containing the prompt's parameters, validated
    ///   against the schema returned by `arguments()`.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The rendered prompt text
    /// * `Err(String)` - An error message if rendering fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use serde_json::Value;
    ///
    /// # async fn example(arguments: &Value) -> Result<String, String> {
    /// let name = arguments
    ///     .get("name")
    ///     .and_then(|v| v.as_str())
    ///     .unwrap_or("World");
    /// Ok(format!("Hello, {}!", name))
    /// # }
    /// ```
    async fn render(&self, arguments: &Value) -> Result<String, String>;
}


