//! Example MCP server with file system resources.
//!
//! This example demonstrates:
//! - Reading files from the file system
//! - Handling file errors gracefully
//! - Different MIME types based on file extensions

use async_trait::async_trait;
use axum_mcp::{McpServer, Resource};
use std::path::PathBuf;

/// File system resource that reads files from disk.
struct FileSystemResource {
    path: PathBuf,
    mime_type: String,
}

impl FileSystemResource {
    fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let mime_type = determine_mime_type(&path);
        Self { path, mime_type }
    }
}

fn determine_mime_type(path: &std::path::Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "txt" => "text/plain",
            "json" => "application/json",
            "yaml" | "yml" => "text/yaml",
            "md" => "text/markdown",
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "rs" => "text/x-rust",
            "py" => "text/x-python",
            "go" => "text/x-go",
            _ => "application/octet-stream",
        })
        .unwrap_or("text/plain")
        .to_string()
}

#[async_trait]
impl Resource for FileSystemResource {
    fn name(&self) -> &str {
        "filesystem"
    }

    fn description(&self) -> &str {
        "File system resource"
    }

    fn mime_type(&self) -> &str {
        &self.mime_type
    }

    async fn read(&self) -> Result<String, String> {
        tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| format!("Failed to read file '{}': {}", self.path.display(), e))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let mut server = McpServer::new();

    // Register file system resources
    // Note: In a real application, you'd want to validate paths and restrict access
    if PathBuf::from("README.md").exists() {
        server.register_resource("file:///README.md", FileSystemResource::new("README.md"))?;
    }

    if PathBuf::from("Cargo.toml").exists() {
        server.register_resource("file:///Cargo.toml", FileSystemResource::new("Cargo.toml"))?;
    }

    println!("Starting file system MCP server on http://127.0.0.1:8080");
    println!("Registered resources:");
    println!("  - file:///README.md");
    println!("  - file:///Cargo.toml");

    server.serve("127.0.0.1:8080").await?;

    Ok(())
}
