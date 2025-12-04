//! Resource definitions for MCP.
//!
//! Resources are data sources that agents can access. Unlike tools, resources are
//! read-only and represent file-like data (e.g., file contents, API responses, database records).
//!
//! # Example
//!
//! ```rust,no_run
//! use mcp_axum::Resource;
//! use async_trait::async_trait;
//!
//! struct HelloResource;
//!
//! #[async_trait]
//! impl Resource for HelloResource {
//!     fn name(&self) -> &str {
//!         "Hello World Resource"
//!     }
//!
//!     fn description(&self) -> &str {
//!         "A simple hello world resource"
//!     }
//!
//!     fn mime_type(&self) -> &str {
//!         "text/plain"
//!     }
//!
//!     async fn read(&self) -> Result<String, String> {
//!         Ok("Hello, World!".to_string())
//!     }
//! }
//! ```

use async_trait::async_trait;

/// A resource that can be accessed by MCP clients.
///
/// Resources represent read-only data sources that agents can access. They are
/// identified by URIs (e.g., `file://path/to/file`, `api://endpoint`) and have
/// associated MIME types for content negotiation.
///
/// # MIME Types
///
/// Common MIME types:
/// - `text/plain` - Plain text
/// - `text/markdown` - Markdown content
/// - `application/json` - JSON data
/// - `text/html` - HTML content
#[async_trait]
pub trait Resource: Send + Sync {
    /// Get the resource's display name.
    ///
    /// This is a human-readable name shown to agents when listing available resources.
    fn name(&self) -> &str;

    /// Get the resource's description.
    ///
    /// Describe what data this resource provides and when agents should access it.
    fn description(&self) -> &str;

    /// Get the resource's MIME type.
    ///
    /// This indicates the format of the content returned by `read()`.
    /// See [IANA Media Types](https://www.iana.org/assignments/media-types/) for standard types.
    fn mime_type(&self) -> &str;

    /// Read the resource's content.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The resource content as a string
    /// * `Err(String)` - An error message if the resource cannot be read
    ///
    /// # Note
    ///
    /// The content is returned as a string regardless of MIME type. For binary data,
    /// consider base64 encoding or using a text-based representation.
    async fn read(&self) -> Result<String, String>;
}


