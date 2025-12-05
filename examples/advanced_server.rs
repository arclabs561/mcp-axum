//! Advanced MCP server example with multiple tools, resources, and prompts.
//!
//! This example demonstrates:
//! - Multiple tool implementations
//! - File system resources
//! - Dynamic prompt templates
//! - Error handling
//! - Custom middleware usage

use async_trait::async_trait;
use axum_mcp::{McpServer, Prompt, Resource, Tool};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Tools
// ============================================================================

/// Calculator tool that performs basic arithmetic operations.
struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn description(&self) -> &str {
        "Performs basic arithmetic operations (add, subtract, multiply, divide)"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "The arithmetic operation to perform"
                },
                "a": {
                    "type": "number",
                    "description": "First number"
                },
                "b": {
                    "type": "number",
                    "description": "Second number"
                }
            },
            "required": ["operation", "a", "b"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let operation = arguments
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'operation' parameter".to_string())?;

        let a = arguments
            .get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| "Missing or invalid 'a' parameter".to_string())?;

        let b = arguments
            .get("b")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| "Missing or invalid 'b' parameter".to_string())?;

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                a / b
            }
            _ => return Err(format!("Unknown operation: {}", operation)),
        };

        Ok(json!({
            "result": result,
            "operation": operation,
            "a": a,
            "b": b
        }))
    }
}

/// String manipulation tool.
struct StringTool;

#[async_trait]
impl Tool for StringTool {
    fn description(&self) -> &str {
        "Manipulates strings (reverse, uppercase, lowercase, length)"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["reverse", "uppercase", "lowercase", "length"],
                    "description": "The string operation to perform"
                },
                "text": {
                    "type": "string",
                    "description": "The input text"
                }
            },
            "required": ["operation", "text"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let operation = arguments
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'operation' parameter".to_string())?;

        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'text' parameter".to_string())?;

        let result = match operation {
            "reverse" => text.chars().rev().collect::<String>(),
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "length" => return Ok(json!({ "length": text.chars().count() })),
            _ => return Err(format!("Unknown operation: {}", operation)),
        };

        Ok(json!({ "result": result }))
    }
}

/// Weather lookup tool (mock implementation).
struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn description(&self) -> &str {
        "Gets weather information for a location"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name or location"
                },
                "units": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "default": "celsius",
                    "description": "Temperature units"
                }
            },
            "required": ["location"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let location = arguments
            .get("location")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'location' parameter".to_string())?;

        let units = arguments
            .get("units")
            .and_then(|v| v.as_str())
            .unwrap_or("celsius");

        // Mock weather data
        let temp = match location.to_lowercase().as_str() {
            "london" => {
                if units == "celsius" {
                    15.0
                } else {
                    59.0
                }
            }
            "new york" => {
                if units == "celsius" {
                    22.0
                } else {
                    72.0
                }
            }
            "tokyo" => {
                if units == "celsius" {
                    25.0
                } else {
                    77.0
                }
            }
            _ => {
                if units == "celsius" {
                    20.0
                } else {
                    68.0
                }
            }
        };

        Ok(json!({
            "location": location,
            "temperature": temp,
            "units": units,
            "condition": "partly cloudy",
            "humidity": 65,
            "wind_speed": 10
        }))
    }
}

// ============================================================================
// Resources
// ============================================================================

/// In-memory key-value store resource.
struct KeyValueResource {
    data: Arc<std::sync::Mutex<HashMap<String, String>>>,
}

impl KeyValueResource {
    fn new() -> Self {
        Self {
            data: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Resource for KeyValueResource {
    fn name(&self) -> &str {
        "kv_store"
    }

    fn description(&self) -> &str {
        "In-memory key-value store"
    }

    fn mime_type(&self) -> &str {
        "application/json"
    }

    async fn read(&self) -> Result<String, String> {
        let data = self.data.lock().unwrap();
        let json = json!(data.clone());
        serde_json::to_string(&json).map_err(|e| format!("Failed to serialize: {}", e))
    }
}

/// Static text resource.
struct StaticResource {
    content: String,
}

impl StaticResource {
    fn new(content: String) -> Self {
        Self { content }
    }
}

#[async_trait]
impl Resource for StaticResource {
    fn name(&self) -> &str {
        "static"
    }

    fn description(&self) -> &str {
        "Static text resource"
    }

    fn mime_type(&self) -> &str {
        "text/plain"
    }

    async fn read(&self) -> Result<String, String> {
        Ok(self.content.clone())
    }
}

// ============================================================================
// Prompts
// ============================================================================

/// Code review prompt template.
struct CodeReviewPrompt;

#[async_trait]
impl Prompt for CodeReviewPrompt {
    fn description(&self) -> &str {
        "Generates a code review prompt for the given code"
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
                "description": "What to focus on (performance, security, style, etc.)",
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
            "Please review the following code with a focus on {}:\n\n```\n{}\n```\n\nProvide feedback on:\n- Code quality\n- Potential bugs\n- Performance improvements\n- Best practices",
            focus, code
        ))
    }
}

/// Summarization prompt template.
struct SummarizePrompt;

#[async_trait]
impl Prompt for SummarizePrompt {
    fn description(&self) -> &str {
        "Generates a summarization prompt for the given text"
    }

    fn arguments(&self) -> Value {
        json!([
            {
                "name": "text",
                "description": "The text to summarize",
                "required": true
            },
            {
                "name": "max_length",
                "description": "Maximum length of summary in words",
                "required": false
            }
        ])
    }

    async fn render(&self, arguments: &Value) -> Result<String, String> {
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'text' argument".to_string())?;

        let max_length = arguments
            .get("max_length")
            .and_then(|v| v.as_u64())
            .unwrap_or(100);

        Ok(format!(
            "Please summarize the following text in approximately {} words:\n\n{}",
            max_length, text
        ))
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create server
    let mut server = McpServer::new();

    // Register tools
    server.register_tool("calculator", CalculatorTool)?;
    server.register_tool("string_ops", StringTool)?;
    server.register_tool("weather", WeatherTool)?;

    // Register resources
    server.register_resource("kv://store", KeyValueResource::new())?;
    server.register_resource(
        "static://welcome",
        StaticResource::new("Welcome to the Advanced MCP Server!".to_string()),
    )?;

    // Register prompts
    server.register_prompt("code_review", CodeReviewPrompt)?;
    server.register_prompt("summarize", SummarizePrompt)?;

    // Start server
    println!("Starting advanced MCP server on http://127.0.0.1:8080");
    println!("Available endpoints:");
    println!("  GET  /health");
    println!("  GET  /tools/list");
    println!("  POST /tools/call");
    println!("  GET  /resources/list");
    println!("  POST /resources/read");
    println!("  GET  /prompts/list");
    println!("  POST /prompts/get");

    server.serve("127.0.0.1:8080").await?;

    Ok(())
}
