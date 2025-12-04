//! Configuration options for MCP server.

use std::time::Duration;

/// Configuration for MCP server behavior.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Timeout for tool execution (default: 30 seconds).
    pub tool_timeout: Duration,
    /// Timeout for resource reads (default: 30 seconds).
    pub resource_timeout: Duration,
    /// Timeout for prompt rendering (default: 30 seconds).
    pub prompt_timeout: Duration,
    /// Maximum request body size in bytes (default: 10MB).
    pub max_body_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            tool_timeout: Duration::from_secs(30),
            resource_timeout: Duration::from_secs(30),
            prompt_timeout: Duration::from_secs(30),
            max_body_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

impl ServerConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set tool execution timeout.
    pub fn with_tool_timeout(mut self, timeout: Duration) -> Self {
        self.tool_timeout = timeout;
        self
    }

    /// Set resource read timeout.
    pub fn with_resource_timeout(mut self, timeout: Duration) -> Self {
        self.resource_timeout = timeout;
        self
    }

    /// Set prompt render timeout.
    pub fn with_prompt_timeout(mut self, timeout: Duration) -> Self {
        self.prompt_timeout = timeout;
        self
    }

    /// Set maximum request body size.
    pub fn with_max_body_size(mut self, size: usize) -> Self {
        self.max_body_size = size;
        self
    }
}
