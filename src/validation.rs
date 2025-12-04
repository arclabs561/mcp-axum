//! Validation utilities for MCP server.

/// Validates a tool name according to MCP specification.
///
/// Tool names SHOULD:
/// - Be between 1 and 128 characters (inclusive)
/// - Be case-sensitive
/// - Contain only: uppercase and lowercase ASCII letters (A-Z, a-z),
///   digits (0-9), underscore (_), hyphen (-), and dot (.)
/// - NOT contain spaces, commas, or other special characters
///
/// # Examples
///
/// ```
/// use mcp_axum::validate_tool_name;
///
/// assert!(validate_tool_name("getUser").is_ok());
/// assert!(validate_tool_name("DATA_EXPORT_v2").is_ok());
/// assert!(validate_tool_name("admin.tools.list").is_ok());
/// assert!(validate_tool_name("invalid name").is_err()); // Contains space
/// assert!(validate_tool_name("").is_err()); // Too short
/// ```
pub fn validate_tool_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Tool name cannot be empty".to_string());
    }

    if name.len() > 128 {
        return Err(format!(
            "Tool name '{}' exceeds maximum length of 128 characters",
            name
        ));
    }

    // Check for invalid characters
    for ch in name.chars() {
        if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' && ch != '.' {
            return Err(format!(
                "Tool name '{}' contains invalid character '{}'. Only A-Z, a-z, 0-9, _, -, and . are allowed",
                name, ch
            ));
        }
    }

    Ok(())
}

/// Validates a resource URI format.
///
/// Basic validation to ensure URI is not empty and has a reasonable format.
pub fn validate_resource_uri(uri: &str) -> Result<(), String> {
    if uri.is_empty() {
        return Err("Resource URI cannot be empty".to_string());
    }

    if uri.len() > 2048 {
        return Err("Resource URI exceeds maximum length of 2048 characters".to_string());
    }

    Ok(())
}

/// Validates a prompt name.
///
/// Uses similar rules to tool names for consistency.
pub fn validate_prompt_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Prompt name cannot be empty".to_string());
    }

    if name.len() > 128 {
        return Err(format!(
            "Prompt name '{}' exceeds maximum length of 128 characters",
            name
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_tool_name_valid() {
        assert!(validate_tool_name("getUser").is_ok());
        assert!(validate_tool_name("DATA_EXPORT_v2").is_ok());
        assert!(validate_tool_name("admin.tools.list").is_ok());
        assert!(validate_tool_name("a").is_ok());
        assert!(validate_tool_name("a".repeat(128).as_str()).is_ok());
    }

    #[test]
    fn test_validate_tool_name_invalid() {
        assert!(validate_tool_name("").is_err());
        assert!(validate_tool_name("invalid name").is_err()); // Space
        assert!(validate_tool_name("invalid,name").is_err()); // Comma
        assert!(validate_tool_name("invalid@name").is_err()); // Special char
        assert!(validate_tool_name("a".repeat(129).as_str()).is_err()); // Too long
    }

    #[test]
    fn test_validate_resource_uri() {
        assert!(validate_resource_uri("file:///path/to/file").is_ok());
        assert!(validate_resource_uri("test://resource").is_ok());
        assert!(validate_resource_uri("").is_err());
    }

    #[test]
    fn test_validate_prompt_name() {
        assert!(validate_prompt_name("greeting").is_ok());
        assert!(validate_prompt_name("").is_err());
    }
}

