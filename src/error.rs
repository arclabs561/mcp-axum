//! Error types for MCP server.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Errors that can occur in an MCP server.
#[derive(Debug, Error)]
pub enum McpError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Tool execution error.
    #[error("Tool error: {0}")]
    Tool(String),

    /// Resource access error.
    #[error("Resource error: {0}")]
    Resource(String),

    /// Prompt rendering error.
    #[error("Prompt error: {0}")]
    Prompt(String),

    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Structured error response for HTTP endpoints.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// HTTP status code.
    pub code: u16,
    /// Error message.
    pub message: String,
    /// Optional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ErrorResponse {
    /// Create a new error response.
    pub fn new(code: u16, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
        }
    }

    /// Create an error response with details.
    pub fn with_details(code: u16, message: String, details: String) -> Self {
        Self {
            code,
            message,
            details: Some(details),
        }
    }
}

/// HTTP endpoint error for handler responses.
#[derive(Debug)]
pub struct HttpError {
    /// HTTP status code.
    pub status: StatusCode,
    /// Error message.
    pub message: String,
    /// Optional details.
    pub details: Option<String>,
}

impl HttpError {
    /// Create a new HTTP error.
    pub fn new(status: StatusCode, message: String) -> Self {
        Self {
            status,
            message,
            details: None,
        }
    }

    /// Create an HTTP error with details.
    pub fn with_details(status: StatusCode, message: String, details: String) -> Self {
        Self {
            status,
            message,
            details: Some(details),
        }
    }

    /// Bad request error.
    pub fn bad_request(message: String) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    /// Not found error.
    pub fn not_found(message: String) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    /// Internal server error.
    pub fn internal(message: String) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            code: self.status.as_u16(),
            message: self.message,
            details: self.details,
        };
        (self.status, Json(body)).into_response()
    }
}
