//! MCP server implementation.

use crate::config::ServerConfig;
use crate::error::{HttpError, McpError};
use crate::prompt::Prompt;
use crate::resource::Resource;
use crate::tool::Tool;
use crate::validation::{validate_prompt_name, validate_resource_uri, validate_tool_name};
use axum::http::{HeaderName, HeaderValue};
use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestId, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};
use uuid::Uuid;

/// An MCP server that handles tools, resources, and prompts.
#[derive(Clone)]
pub struct McpServer {
    tools: HashMap<String, Arc<dyn Tool>>,
    resources: HashMap<String, Arc<dyn Resource>>,
    prompts: HashMap<String, Arc<dyn Prompt>>,
    config: ServerConfig,
}

impl McpServer {
    /// Create a new MCP server with default configuration.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            resources: HashMap::new(),
            prompts: HashMap::new(),
            config: ServerConfig::default(),
        }
    }

    /// Create a new MCP server with custom configuration.
    pub fn with_config(config: ServerConfig) -> Self {
        Self {
            tools: HashMap::new(),
            resources: HashMap::new(),
            prompts: HashMap::new(),
            config,
        }
    }

    /// Get a reference to the server configuration.
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get a mutable reference to the server configuration.
    pub fn config_mut(&mut self) -> &mut ServerConfig {
        &mut self.config
    }

    /// Register a tool.
    ///
    /// Validates the tool name according to MCP specification before registration.
    ///
    /// # Errors
    ///
    /// Returns `McpError::Validation` if the tool name is invalid.
    pub fn register_tool(
        &mut self,
        name: impl Into<String>,
        tool: impl Tool + 'static,
    ) -> Result<(), McpError> {
        let name = name.into();
        validate_tool_name(&name)
            .map_err(|e| McpError::Validation(format!("Invalid tool name '{}': {}", name, e)))?;
        self.tools.insert(name, Arc::new(tool));
        Ok(())
    }

    /// Register a resource.
    ///
    /// Validates the resource URI before registration.
    ///
    /// # Errors
    ///
    /// Returns `McpError::Validation` if the resource URI is invalid.
    pub fn register_resource(
        &mut self,
        name: impl Into<String>,
        resource: impl Resource + 'static,
    ) -> Result<(), McpError> {
        let name = name.into();
        validate_resource_uri(&name)
            .map_err(|e| McpError::Validation(format!("Invalid resource URI '{}': {}", name, e)))?;
        self.resources.insert(name, Arc::new(resource));
        Ok(())
    }

    /// Register a prompt.
    ///
    /// Validates the prompt name before registration.
    ///
    /// # Errors
    ///
    /// Returns `McpError::Validation` if the prompt name is invalid.
    pub fn register_prompt(
        &mut self,
        name: impl Into<String>,
        prompt: impl Prompt + 'static,
    ) -> Result<(), McpError> {
        let name = name.into();
        validate_prompt_name(&name)
            .map_err(|e| McpError::Validation(format!("Invalid prompt name '{}': {}", name, e)))?;
        self.prompts.insert(name, Arc::new(prompt));
        Ok(())
    }

    /// Register a tool using builder pattern (chainable).
    ///
    /// This method allows chaining multiple registrations together.
    ///
    /// # Errors
    ///
    /// Returns `McpError::Validation` if the tool name is invalid.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use mcp_axum::{McpServer, Tool};
    /// # use async_trait::async_trait;
    /// # use serde_json::Value;
    /// # struct EchoTool;
    /// # #[async_trait]
    /// # impl Tool for EchoTool {
    /// #     fn description(&self) -> &str { "echo" }
    /// #     fn schema(&self) -> Value { Value::Null }
    /// #     async fn call(&self, _: &Value) -> Result<Value, String> {
    /// #         Ok(Value::Null)
    /// #     }
    /// # }
    /// let server = McpServer::new()
    ///     .tool("echo", EchoTool)?;
    /// # Ok::<(), mcp_axum::McpError>(())
    /// ```
    pub fn tool(
        mut self,
        name: impl Into<String>,
        tool: impl Tool + 'static,
    ) -> Result<Self, McpError> {
        self.register_tool(name, tool)?;
        Ok(self)
    }

    /// Register a resource using builder pattern (chainable).
    ///
    /// This method allows chaining multiple registrations together.
    ///
    /// # Errors
    ///
    /// Returns `McpError::Validation` if the resource URI is invalid.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use mcp_axum::{McpServer, Resource};
    /// # use async_trait::async_trait;
    /// # struct HelloResource;
    /// # #[async_trait]
    /// # impl Resource for HelloResource {
    /// #     fn name(&self) -> &str { "hello" }
    /// #     fn description(&self) -> &str { "hello" }
    /// #     fn mime_type(&self) -> &str { "text/plain" }
    /// #     async fn read(&self) -> Result<String, String> { Ok("hello".to_string()) }
    /// # }
    /// let server = McpServer::new()
    ///     .resource("hello://world", HelloResource)?;
    /// # Ok::<(), mcp_axum::McpError>(())
    /// ```
    pub fn resource(
        mut self,
        name: impl Into<String>,
        resource: impl Resource + 'static,
    ) -> Result<Self, McpError> {
        self.register_resource(name, resource)?;
        Ok(self)
    }

    /// Register a prompt using builder pattern (chainable).
    ///
    /// This method allows chaining multiple registrations together.
    ///
    /// # Errors
    ///
    /// Returns `McpError::Validation` if the prompt name is invalid.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use mcp_axum::{McpServer, Prompt};
    /// # use async_trait::async_trait;
    /// # use serde_json::Value;
    /// # struct GreetingPrompt;
    /// # #[async_trait]
    /// # impl Prompt for GreetingPrompt {
    /// #     fn description(&self) -> &str { "greeting" }
    /// #     fn arguments(&self) -> Value { Value::Null }
    /// #     async fn render(&self, _: &Value) -> Result<String, String> {
    /// #         Ok("hello".to_string())
    /// #     }
    /// # }
    /// let server = McpServer::new()
    ///     .prompt("greeting", GreetingPrompt)?;
    /// # Ok::<(), mcp_axum::McpError>(())
    /// ```
    pub fn prompt(
        mut self,
        name: impl Into<String>,
        prompt: impl Prompt + 'static,
    ) -> Result<Self, McpError> {
        self.register_prompt(name, prompt)?;
        Ok(self)
    }

    /// Build the Axum router.
    ///
    /// Includes middleware for:
    /// - Request tracing and logging
    /// - Request ID generation
    /// - CORS support
    /// - Request body size limits (10MB default)
    pub fn router(self) -> Router {
        let state = Arc::new(self);
        Router::new()
            .route("/health", get(health))
            .route("/tools/list", get(list_tools))
            .route("/tools/call", post(call_tool))
            .route("/resources/list", get(list_resources))
            .route("/resources/read", post(read_resource))
            .route("/prompts/list", get(list_prompts))
            .route("/prompts/get", post(get_prompt))
            .layer(
                ServiceBuilder::new()
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(|request: &axum::http::Request<_>| {
                                let request_id = request
                                    .headers()
                                    .get("x-request-id")
                                    .and_then(|v| v.to_str().ok())
                                    .unwrap_or("unknown");
                                tracing::info_span!(
                                    "http_request",
                                    method = %request.method(),
                                    uri = %request.uri(),
                                    request_id = %request_id,
                                )
                            })
                            .on_request(
                                |_request: &axum::http::Request<_>, _span: &tracing::Span| {
                                    tracing::debug!("request started");
                                },
                            )
                            .on_response(
                                |_response: &axum::http::Response<_>,
                                 latency: std::time::Duration,
                                 _span: &tracing::Span| {
                                    tracing::debug!(latency = ?latency, "request completed");
                                },
                            )
                            .on_failure(
                                |_error: tower_http::classify::ServerErrorsFailureClass,
                                 _latency: std::time::Duration,
                                 _span: &tracing::Span| {
                                    tracing::error!("request failed");
                                },
                            ),
                    )
                    .layer(SetRequestIdLayer::new(
                        HeaderName::from_static("x-request-id"),
                        UuidRequestId,
                    ))
                    .layer(RequestBodyLimitLayer::new(state.config.max_body_size))
                    .layer(CorsLayer::permissive()),
            )
            .with_state(state)
    }

    /// Start the server.
    pub async fn serve(self, addr: &str) -> Result<(), McpError> {
        self.serve_with_shutdown(addr, std::future::pending()).await
    }

    /// Start the server with graceful shutdown support.
    ///
    /// The server will shut down when the provided shutdown signal completes.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mcp_axum::McpServer;
    /// use tokio::signal;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let server = McpServer::new();
    /// let shutdown = async {
    ///     signal::ctrl_c()
    ///         .await
    ///         .expect("failed to install CTRL+C signal handler");
    /// };
    /// server.serve_with_shutdown("127.0.0.1:8080", shutdown).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn serve_with_shutdown<F>(self, addr: &str, shutdown: F) -> Result<(), McpError>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let listener = TcpListener::bind(addr).await?;
        let app = self.router();
        tracing::info!("MCP server listening on {}", addr);
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await?;
        tracing::info!("MCP server shutting down gracefully");
        Ok(())
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

async fn health(State(server): State<Arc<McpServer>>) -> Json<Value> {
    let tool_count = server.tools.len();
    let resource_count = server.resources.len();
    let prompt_count = server.prompts.len();

    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "tools": tool_count,
        "resources": resource_count,
        "prompts": prompt_count,
    }))
}

async fn list_tools(State(server): State<Arc<McpServer>>) -> Json<Value> {
    let tools: Vec<Value> = server
        .tools
        .iter()
        .map(|(name, tool)| {
            let description = tool.description().to_string();
            let schema = tool.schema();
            serde_json::json!({
                "name": name,
                "description": description,
                "inputSchema": schema,
            })
        })
        .collect();
    Json(serde_json::json!({ "tools": tools }))
}

async fn call_tool(
    State(server): State<Arc<McpServer>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, HttpError> {
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HttpError::bad_request("Missing 'name' field in request".to_string()))?;

    // Validate tool name format
    validate_tool_name(name)
        .map_err(|e| HttpError::bad_request(format!("Invalid tool name: {}", e)))?;

    let arguments = payload
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let tool = server
        .tools
        .get(name)
        .ok_or_else(|| HttpError::not_found(format!("Tool '{}' not found", name)))?;

    // Validate arguments against tool schema
    let schema = tool.schema();
    let compiled = jsonschema::JSONSchema::compile(&schema).map_err(|e| {
        tracing::warn!("Failed to compile tool schema for '{}': {}", name, e);
        HttpError::internal("Invalid tool schema configuration".to_string())
    })?;

    let validation_result = compiled.validate(&arguments);
    if let Err(errors) = validation_result {
        let error_messages: Vec<String> = errors
            .map(|e| {
                let path = if e.instance_path.to_string().is_empty() {
                    "root".to_string()
                } else {
                    e.instance_path.to_string()
                };
                format!("{}: {}", path, e)
            })
            .collect();
        tracing::debug!(
            "Schema validation failed for tool '{}' with arguments {:?}: {:?}",
            name,
            arguments,
            error_messages
        );
        return Err(HttpError::bad_request(format!(
            "Arguments for tool '{}' failed schema validation: {}",
            name,
            error_messages.join(", ")
        )));
    }

    // Execute tool with configured timeout
    let timeout_duration = server.config.tool_timeout;
    let result = tokio::time::timeout(timeout_duration, tool.call(&arguments)).await;

    match result {
        Ok(Ok(result_value)) => {
            let text = serde_json::to_string(&result_value).map_err(|e| {
                tracing::error!("Failed to serialize tool result: {}", e);
                HttpError::internal("Failed to serialize tool result".to_string())
            })?;
            Ok(Json(serde_json::json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            })))
        }
        Ok(Err(e)) => {
            tracing::error!("Tool execution error: {}", e);
            Err(HttpError::internal(format!("Tool execution failed: {}", e)))
        }
        Err(_) => {
            tracing::warn!(
                "Tool '{}' execution timed out after {:?}",
                name,
                timeout_duration
            );
            Err(HttpError::internal(format!(
                "Tool '{}' execution timed out after {:?}",
                name, timeout_duration
            )))
        }
    }
}

async fn list_resources(State(server): State<Arc<McpServer>>) -> Json<Value> {
    let resources: Vec<Value> = server
        .resources
        .iter()
        .map(|(name, resource)| {
            let resource_name = resource.name().to_string();
            let description = resource.description().to_string();
            let mime_type = resource.mime_type().to_string();
            serde_json::json!({
                "uri": name,
                "name": resource_name,
                "description": description,
                "mimeType": mime_type,
            })
        })
        .collect();
    Json(serde_json::json!({ "resources": resources }))
}

async fn read_resource(
    State(server): State<Arc<McpServer>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, HttpError> {
    let uri = payload
        .get("uri")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HttpError::bad_request("Missing 'uri' field in request".to_string()))?;

    // Validate URI format
    validate_resource_uri(uri)
        .map_err(|e| HttpError::bad_request(format!("Invalid resource URI: {}", e)))?;

    let resource = server
        .resources
        .get(uri)
        .ok_or_else(|| HttpError::not_found(format!("Resource '{}' not found", uri)))?;

    // Read resource with configured timeout
    let timeout_duration = server.config.resource_timeout;
    let mime_type = resource.mime_type().to_string();
    let read_result = tokio::time::timeout(timeout_duration, resource.read()).await;

    match read_result {
        Ok(Ok(content)) => Ok(Json(serde_json::json!({
            "contents": [{
                "uri": uri,
                "mimeType": mime_type,
                "text": content
            }]
        }))),
        Ok(Err(e)) => {
            tracing::error!("Resource read error: {}", e);
            Err(HttpError::internal(format!("Resource read failed: {}", e)))
        }
        Err(_) => {
            tracing::warn!(
                "Resource '{}' read timed out after {:?}",
                uri,
                timeout_duration
            );
            Err(HttpError::internal(format!(
                "Resource '{}' read timed out after {:?}",
                uri, timeout_duration
            )))
        }
    }
}

async fn list_prompts(State(server): State<Arc<McpServer>>) -> Json<Value> {
    let prompts: Vec<Value> = server
        .prompts
        .iter()
        .map(|(name, prompt)| {
            let description = prompt.description().to_string();
            let arguments = prompt.arguments();
            serde_json::json!({
                "name": name,
                "description": description,
                "arguments": arguments,
            })
        })
        .collect();
    Json(serde_json::json!({ "prompts": prompts }))
}

/// Request ID generator using UUID v4.
#[derive(Clone, Default)]
struct UuidRequestId;

impl MakeRequestId for UuidRequestId {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string();
        HeaderValue::from_str(&request_id).ok().map(RequestId::new)
    }
}

async fn get_prompt(
    State(server): State<Arc<McpServer>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, HttpError> {
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HttpError::bad_request("Missing 'name' field in request".to_string()))?;

    // Validate prompt name
    validate_prompt_name(name)
        .map_err(|e| HttpError::bad_request(format!("Invalid prompt name: {}", e)))?;

    let arguments = payload
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let prompt = server
        .prompts
        .get(name)
        .ok_or_else(|| HttpError::not_found(format!("Prompt '{}' not found", name)))?;

    // Render prompt with configured timeout
    let timeout_duration = server.config.prompt_timeout;
    let render_result = tokio::time::timeout(timeout_duration, prompt.render(&arguments)).await;

    match render_result {
        Ok(Ok(content)) => Ok(Json(serde_json::json!({
            "messages": [{
                "role": "user",
                "content": {
                    "type": "text",
                    "text": content
                }
            }]
        }))),
        Ok(Err(e)) => {
            tracing::error!("Prompt render error: {}", e);
            Err(HttpError::internal(format!("Prompt render failed: {}", e)))
        }
        Err(_) => {
            tracing::warn!(
                "Prompt '{}' render timed out after {:?}",
                name,
                timeout_duration
            );
            Err(HttpError::internal(format!(
                "Prompt '{}' render timed out after {:?}",
                name, timeout_duration
            )))
        }
    }
}
