//! Example MCP server with database tools.
//!
//! This example demonstrates:
//! - Database query tools
//! - Transaction handling
//! - Error handling for database operations
//! - Using in-memory SQLite for demonstration

use async_trait::async_trait;
use mcp_axum::{McpServer, Tool};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// In-memory database for demonstration purposes.
/// In production, you'd use a real database like PostgreSQL, MySQL, etc.
struct Database {
    data: Arc<std::sync::Mutex<HashMap<String, Value>>>,
}

impl Database {
    fn new() -> Self {
        Self {
            data: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    fn insert(&self, key: String, value: Value) -> Result<(), String> {
        let mut data = self.data.lock().unwrap();
        data.insert(key, value);
        Ok(())
    }

    fn get(&self, key: &str) -> Option<Value> {
        let data = self.data.lock().unwrap();
        data.get(key).cloned()
    }

    fn delete(&self, key: &str) -> bool {
        let mut data = self.data.lock().unwrap();
        data.remove(key).is_some()
    }

    fn list_keys(&self) -> Vec<String> {
        let data = self.data.lock().unwrap();
        data.keys().cloned().collect()
    }
}

/// Database insert tool.
struct DbInsertTool {
    db: Arc<Database>,
}

#[async_trait]
impl Tool for DbInsertTool {
    fn description(&self) -> &str {
        "Inserts a key-value pair into the database"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "The key to insert"
                },
                "value": {
                    "type": ["string", "number", "boolean", "object", "array"],
                    "description": "The value to insert"
                }
            },
            "required": ["key", "value"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let key = arguments
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'key' parameter".to_string())?
            .to_string();

        let value = arguments
            .get("value")
            .ok_or_else(|| "Missing 'value' parameter".to_string())?
            .clone();

        self.db.insert(key.clone(), value.clone())?;

        Ok(json!({
            "status": "inserted",
            "key": key,
            "value": value
        }))
    }
}

/// Database get tool.
struct DbGetTool {
    db: Arc<Database>,
}

#[async_trait]
impl Tool for DbGetTool {
    fn description(&self) -> &str {
        "Retrieves a value from the database by key"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "The key to retrieve"
                }
            },
            "required": ["key"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let key = arguments
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'key' parameter".to_string())?;

        match self.db.get(key) {
            Some(value) => Ok(json!({
                "status": "found",
                "key": key,
                "value": value
            })),
            None => Ok(json!({
                "status": "not_found",
                "key": key
            })),
        }
    }
}

/// Database delete tool.
struct DbDeleteTool {
    db: Arc<Database>,
}

#[async_trait]
impl Tool for DbDeleteTool {
    fn description(&self) -> &str {
        "Deletes a key-value pair from the database"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "The key to delete"
                }
            },
            "required": ["key"]
        })
    }

    async fn call(&self, arguments: &Value) -> Result<Value, String> {
        let key = arguments
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'key' parameter".to_string())?;

        let deleted = self.db.delete(key);

        Ok(json!({
            "status": if deleted { "deleted" } else { "not_found" },
            "key": key
        }))
    }
}

/// Database list keys tool.
struct DbListTool {
    db: Arc<Database>,
}

#[async_trait]
impl Tool for DbListTool {
    fn description(&self) -> &str {
        "Lists all keys in the database"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn call(&self, _arguments: &Value) -> Result<Value, String> {
        let keys = self.db.list_keys();
        Ok(json!({
            "keys": keys,
            "count": keys.len()
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let db = Arc::new(Database::new());
    let mut server = McpServer::new();

    // Register database tools
    server.register_tool(
        "db_insert",
        DbInsertTool {
            db: Arc::clone(&db),
        },
    )?;
    server.register_tool(
        "db_get",
        DbGetTool {
            db: Arc::clone(&db),
        },
    )?;
    server.register_tool(
        "db_delete",
        DbDeleteTool {
            db: Arc::clone(&db),
        },
    )?;
    server.register_tool("db_list", DbListTool { db })?;

    println!("Starting database MCP server on http://127.0.0.1:8080");
    println!("Available tools:");
    println!("  - db_insert: Insert a key-value pair");
    println!("  - db_get: Get a value by key");
    println!("  - db_delete: Delete a key-value pair");
    println!("  - db_list: List all keys");

    server.serve("127.0.0.1:8080").await?;

    Ok(())
}
