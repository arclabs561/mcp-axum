//! JSON Schema generation from Rust docstrings.
//!
//! This module provides utilities for extracting JSON Schema from Rust docstrings,
//! enabling automatic API documentation generation from code comments.
//!
//! # Supported Docstring Format
//!
//! The parser expects docstrings in this format:
//!
//! ```text
//! /// Function description.
//! ///
//! /// # Arguments
//! /// * `param1` - Description of param1 (type: string)
//! /// * `param2` - Description of param2 (type: number, default: 10)
//! /// * `param3` - Optional parameter (type: boolean, default: false)
//! ///
//! /// # Returns
//! /// Description of return value
//! ```
//!
//! # Type Mapping
//!
//! Rust types are mapped to JSON Schema types:
//!
//! - `String`, `&str` → `"string"`
//! - `usize`, `u32`, `u64`, `i32`, `i64` → `"integer"`
//! - `f32`, `f64` → `"number"`
//! - `bool` → `"boolean"`
//! - Everything else → `"object"`
//!
//! # Default Values
//!
//! Default values are parsed from the `(default: value)` annotation:
//!
//! - Numbers: `(default: 42)` or `(default: 3.14)`
//! - Booleans: `(default: true)` or `(default: false)`
//! - Strings: `(default: "hello")` or `(default: 'world')`
//!
//! Parameters with defaults are automatically marked as optional (not in `required` array).

use serde_json::Value;

/// Extract JSON Schema from a Rust docstring.
///
/// This function parses docstrings in the format:
///
/// ```text
/// /// Function description.
/// ///
/// /// # Arguments
/// /// * `param1` - Description of param1 (type: string)
/// /// * `param2` - Description of param2 (type: number, default: 10)
/// ///
/// /// # Returns
/// /// Description of return value (type: object)
/// ```
pub fn extract_schema_from_docstring(docstring: &str) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    // Look for # Arguments section
    let args_section = if let Some(start) = docstring.find("# Arguments") {
        let section = &docstring[start..];
        if let Some(end) = section.find("\n\n") {
            &section[..end]
        } else {
            section
        }
    } else {
        return serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        });
    };

    // Parse each argument line: * `param` - description (type: type, default: value)
    for line in args_section.lines() {
        let line = line.trim();
        if !line.starts_with('*') {
            continue;
        }

        // Extract parameter name from `param`
        let param_start = if let Some(start) = line.find('`') {
            start + 1
        } else {
            continue;
        };
        let param_end = if let Some(end) = line[param_start..].find('`') {
            param_start + end
        } else {
            continue;
        };
        let param_name = &line[param_start..param_end];

        // Extract description and type/default info
        let rest = &line[param_end + 1..];
        let mut param_type = "object";
        let mut default_value: Option<Value> = None;

        // Look for parentheses group that may contain type and/or default
        if let Some(paren_start) = rest.find('(') {
            if let Some(paren_end) = rest[paren_start..].find(')') {
                let paren_content = &rest[paren_start + 1..paren_start + paren_end];
                
                // Extract type from (type: ...) pattern
                if let Some(type_start) = paren_content.find("type:") {
                    let type_content = &paren_content[type_start + 5..];
                    // Find comma or end of parentheses for type
                    let type_end = type_content.find(',').unwrap_or(type_content.len());
                    let type_str = type_content[..type_end].trim();
                    param_type = match type_str {
                        "string" | "String" | "&str" => "string",
                        "number" | "f32" | "f64" => "number",
                        "integer" | "usize" | "u32" | "u64" | "i32" | "i64" => "integer",
                        "boolean" | "bool" => "boolean",
                        _ => "object",
                    };
                }
                
                // Extract default from (default: ...) pattern
                if let Some(default_start) = paren_content.find("default:") {
                    let default_content = &paren_content[default_start + 8..];
                    // Find comma or end for default value
                    let default_end = default_content.find(',').unwrap_or(default_content.len());
                    let default_str = default_content[..default_end].trim();
                    // Try to parse as integer first (for integer types)
                    if let Ok(int) = default_str.parse::<i64>() {
                        default_value = Some(serde_json::json!(int));
                    } else if let Ok(num) = default_str.parse::<f64>() {
                        default_value = Some(serde_json::json!(num));
                    } else if default_str == "true" || default_str == "false" {
                        default_value = Some(serde_json::json!(default_str == "true"));
                    } else {
                        // Treat as string (remove quotes if present)
                        let clean_str = default_str.trim_matches('"').trim_matches('\'');
                        default_value = Some(serde_json::json!(clean_str));
                    }
                }
            }
        }
        
        // If no default, parameter is required
        if default_value.is_none() {
            required.push(param_name.to_string());
        }

        // Create property schema
        let mut prop = serde_json::Map::new();
        prop.insert("type".to_string(), serde_json::Value::String(param_type.to_string()));

        // Add description if available
        if let Some(desc_start) = rest.find('-') {
            let desc = rest[desc_start + 1..].trim();
            // Remove type and default annotations from description
            let clean_desc = desc
                .split("(type:")
                .next()
                .unwrap_or(desc)
                .split("(default:")
                .next()
                .unwrap_or(desc)
                .trim();
            if !clean_desc.is_empty() {
                prop.insert("description".to_string(), serde_json::Value::String(clean_desc.to_string()));
            }
        }

        // Add default value if present
        if let Some(default) = default_value {
            prop.insert("default".to_string(), default);
        }

        properties.insert(param_name.to_string(), serde_json::Value::Object(prop));
    }

    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

/// Generate JSON Schema from a Rust type.
pub fn schema_from_type(ty: &str) -> Value {
    match ty {
        "String" | "&str" => serde_json::json!({"type": "string"}),
        "usize" | "u32" | "u64" | "i32" | "i64" => serde_json::json!({"type": "integer"}),
        "f32" | "f64" => serde_json::json!({"type": "number"}),
        "bool" => serde_json::json!({"type": "boolean"}),
        _ => serde_json::json!({"type": "object"}),
    }
}

