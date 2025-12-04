//! Structured error types for tool execution.
//!
//! Provides more detailed error information than simple `String` errors.

use serde::Serialize;
use thiserror::Error;

/// Errors that can occur during tool execution.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ToolError {
    /// A required parameter was missing.
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    /// A parameter had an invalid type.
    #[error("Invalid parameter '{param}': expected {expected}, got {got}")]
    InvalidType {
        /// Parameter name.
        param: String,
        /// Expected type.
        expected: String,
        /// Actual type received.
        got: String,
    },

    /// A parameter value was invalid (e.g., out of range).
    #[error("Invalid value for parameter '{param}': {reason}")]
    InvalidValue {
        /// Parameter name.
        param: String,
        /// Reason the value is invalid.
        reason: String,
    },

    /// Tool execution failed.
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Tool execution timed out.
    #[error("Execution timed out after {0} seconds")]
    Timeout(u64),
}

impl ToolError {
    /// Create a missing parameter error.
    pub fn missing_parameter(param: impl Into<String>) -> Self {
        Self::MissingParameter(param.into())
    }

    /// Create an invalid type error.
    pub fn invalid_type(
        param: impl Into<String>,
        expected: impl Into<String>,
        got: impl Into<String>,
    ) -> Self {
        Self::InvalidType {
            param: param.into(),
            expected: expected.into(),
            got: got.into(),
        }
    }

    /// Create an invalid value error.
    pub fn invalid_value(param: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidValue {
            param: param.into(),
            reason: reason.into(),
        }
    }

    /// Create an execution failed error.
    pub fn execution_failed(msg: impl Into<String>) -> Self {
        Self::ExecutionFailed(msg.into())
    }

    /// Create a timeout error.
    pub fn timeout(seconds: u64) -> Self {
        Self::Timeout(seconds)
    }
}

impl From<ToolError> for String {
    fn from(err: ToolError) -> Self {
        err.to_string()
    }
}

/// HTTP status code mapping for tool errors.
impl ToolError {
    /// Get the appropriate HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            ToolError::MissingParameter(_)
            | ToolError::InvalidType { .. }
            | ToolError::InvalidValue { .. } => {
                400 // Bad Request
            }
            ToolError::ExecutionFailed(_) => 500, // Internal Server Error
            ToolError::Timeout(_) => 504,         // Gateway Timeout
        }
    }
}

/// Structured error response for tool errors.
#[derive(Debug, Serialize)]
pub struct ToolErrorResponse {
    /// Error code (HTTP status code).
    pub code: u16,
    /// Error message.
    pub message: String,
    /// Error type.
    pub error_type: String,
    /// Optional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl From<ToolError> for ToolErrorResponse {
    fn from(err: ToolError) -> Self {
        Self {
            code: err.status_code(),
            message: err.to_string(),
            error_type: format!("{:?}", err),
            details: match &err {
                ToolError::InvalidType {
                    param,
                    expected,
                    got,
                } => Some(format!(
                    "Parameter '{}' should be {} but got {}",
                    param, expected, got
                )),
                ToolError::InvalidValue { param, reason } => {
                    Some(format!("Parameter '{}' is invalid: {}", param, reason))
                }
                _ => None,
            },
        }
    }
}
