//! Utility functions for common MCP server patterns.
//!
//! This module provides helpers to reduce boilerplate when working with MCP tools,
//! resources, and prompts.

use serde_json::Value;

/// Extract a required string argument from tool arguments.
///
/// # Errors
///
/// Returns an error if the parameter is missing or not a string.
///
/// # Example
///
/// ```rust,no_run
/// use axum_mcp::extract_string;
/// use serde_json::json;
///
/// let args = json!({"text": "hello"});
/// let text: String = extract_string(&args, "text")?;
/// # Ok::<(), String>(())
/// ```
pub fn extract_string(arguments: &Value, param: &str) -> Result<String, String> {
    arguments
        .get(param)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing required parameter '{}'", param))
}

/// Extract an optional string argument from tool arguments.
///
/// # Example
///
/// ```rust,no_run
/// use axum_mcp::extract_string_opt;
/// use serde_json::json;
///
/// let args = json!({"text": "hello"});
/// let text: Option<String> = extract_string_opt(&args, "text");
/// ```
pub fn extract_string_opt(arguments: &Value, param: &str) -> Option<String> {
    arguments
        .get(param)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Extract a required number argument from tool arguments.
///
/// # Errors
///
/// Returns an error if the parameter is missing or not a number.
///
/// # Example
///
/// ```rust,no_run
/// use axum_mcp::extract_number;
/// use serde_json::json;
///
/// let args = json!({"limit": 10});
/// let limit: f64 = extract_number(&args, "limit")?;
/// # Ok::<(), String>(())
/// ```
pub fn extract_number(arguments: &Value, param: &str) -> Result<f64, String> {
    arguments
        .get(param)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| format!("Missing or invalid number parameter '{}'", param))
}

/// Extract an optional number argument from tool arguments.
pub fn extract_number_opt(arguments: &Value, param: &str) -> Option<f64> {
    arguments.get(param).and_then(|v| v.as_f64())
}

/// Extract a required integer argument from tool arguments.
///
/// # Errors
///
/// Returns an error if the parameter is missing or not an integer.
pub fn extract_integer(arguments: &Value, param: &str) -> Result<i64, String> {
    arguments
        .get(param)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| format!("Missing or invalid integer parameter '{}'", param))
}

/// Extract an optional integer argument from tool arguments.
pub fn extract_integer_opt(arguments: &Value, param: &str) -> Option<i64> {
    arguments.get(param).and_then(|v| v.as_i64())
}

/// Extract a required boolean argument from tool arguments.
///
/// # Errors
///
/// Returns an error if the parameter is missing or not a boolean.
pub fn extract_bool(arguments: &Value, param: &str) -> Result<bool, String> {
    arguments
        .get(param)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| format!("Missing or invalid boolean parameter '{}'", param))
}

/// Extract an optional boolean argument from tool arguments.
pub fn extract_bool_opt(arguments: &Value, param: &str) -> Option<bool> {
    arguments.get(param).and_then(|v| v.as_bool())
}
