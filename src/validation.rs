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
/// use axum_mcp::validate_tool_name;
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

/// Validates a resource URI according to MCP specification.
///
/// Resource URIs SHOULD:
/// - Be between 1 and 2048 characters (inclusive)
/// - Be case-sensitive
/// - Follow URI syntax (e.g., `scheme://path/to/resource`)
/// - Contain a valid URI scheme (e.g., `file://`, `http://`, `https://`)
///
/// # Examples
///
/// ```
/// use axum_mcp::validate_resource_uri;
///
/// assert!(validate_resource_uri("file:///path/to/file.txt").is_ok());
/// assert!(validate_resource_uri("http://example.com/data").is_ok());
/// assert!(validate_resource_uri("https://api.example.com/resource").is_ok());
/// assert!(validate_resource_uri("").is_err()); // Too short
/// assert!(validate_resource_uri("invalid-uri").is_err()); // Not a valid URI
/// ```
pub fn validate_resource_uri(uri: &str) -> Result<(), String> {
    if uri.is_empty() {
        return Err("Resource URI cannot be empty".to_string());
    }

    // Check character count (not byte count) for UTF-8 safety
    let char_count = uri.chars().count();
    if char_count > 2048 {
        return Err(format!(
            "Resource URI '{}' exceeds maximum length of 2048 characters",
            uri
        ));
    }

    // Basic URI scheme validation (e.g., "scheme://...")
    // This ensures the URI has a scheme separator
    if !uri.contains("://") {
        return Err(format!(
            "Resource URI '{}' is not a valid URI (missing scheme, expected format: scheme://path)",
            uri
        ));
    }

    // Validate scheme part (before ://)
    if let Some(scheme_end) = uri.find("://") {
        let scheme = &uri[..scheme_end];
        if scheme.is_empty() {
            return Err(format!(
                "Resource URI '{}' has empty scheme (expected format: scheme://path)",
                uri
            ));
        }
        // Scheme should contain only valid characters (alphanumeric, +, -, .)
        if !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
        {
            return Err(format!(
                "Resource URI '{}' has invalid scheme '{}' (scheme must contain only alphanumeric, +, -, or . characters)",
                uri, scheme
            ));
        }

        // Require non-empty path after scheme://
        let path = &uri[scheme_end + 3..];
        if path.is_empty() {
            return Err(format!(
                "Resource URI '{}' has empty path (expected format: scheme://path)",
                uri
            ));
        }
    }

    Ok(())
}

/// Validates a prompt name according to MCP specification.
///
/// Prompt names SHOULD:
/// - Be between 1 and 128 characters (inclusive)
/// - Be case-sensitive
/// - Contain only: uppercase and lowercase ASCII letters (A-Z, a-z),
///   digits (0-9), underscore (_), hyphen (-), and dot (.)
/// - NOT contain spaces, commas, or other special characters
///
/// This follows the same rules as tool names.
///
/// # Examples
///
/// ```
/// use axum_mcp::validate_prompt_name;
///
/// assert!(validate_prompt_name("summarize_text").is_ok());
/// assert!(validate_prompt_name("CODE_GEN_v1").is_ok());
/// assert!(validate_prompt_name("invalid name").is_err()); // Contains space
/// ```
pub fn validate_prompt_name(name: &str) -> Result<(), String> {
    // Prompt names follow the same validation rules as tool names
    validate_tool_name(name)
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
        assert!(validate_resource_uri("http://example.com/data").is_ok());
        assert!(validate_resource_uri("https://api.example.com/resource").is_ok());
        assert!(validate_resource_uri("test://resource").is_ok());
        assert!(validate_resource_uri("custom+scheme://path").is_ok());
        assert!(validate_resource_uri("").is_err()); // Empty
        assert!(validate_resource_uri("invalid-uri").is_err()); // No scheme
        assert!(validate_resource_uri("://path").is_err()); // Empty scheme
        assert!(validate_resource_uri("invalid@scheme://path").is_err()); // Invalid scheme char
        let long_uri = "a".repeat(2049);
        assert!(validate_resource_uri(&long_uri).is_err()); // Too long
    }

    #[test]
    fn test_validate_prompt_name() {
        assert!(validate_prompt_name("greeting").is_ok());
        assert!(validate_prompt_name("summarize_text").is_ok());
        assert!(validate_prompt_name("CODE_GEN_v1").is_ok());
        assert!(validate_prompt_name("admin.prompts.list").is_ok());
        assert!(validate_prompt_name("").is_err());
        assert!(validate_prompt_name("invalid name").is_err()); // Space
        assert!(validate_prompt_name("invalid,name").is_err()); // Comma
        assert!(validate_prompt_name("a".repeat(129).as_str()).is_err()); // Too long
    }
}
