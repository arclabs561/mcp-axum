//! Edge case tests for validation functions.

use mcp_axum::{validate_prompt_name, validate_resource_uri, validate_tool_name};

#[test]
fn test_tool_name_edge_cases() {
    // Valid edge cases
    assert!(validate_tool_name("a").is_ok()); // Minimum length
    assert!(validate_tool_name(&"a".repeat(128)).is_ok()); // Maximum length
    assert!(validate_tool_name("a_b-c.d").is_ok()); // All allowed characters

    // Invalid edge cases
    assert!(validate_tool_name("").is_err()); // Empty
    assert!(validate_tool_name(&"a".repeat(129)).is_err()); // Too long
    assert!(validate_tool_name("tool name").is_err()); // Space
    assert!(validate_tool_name("tool,name").is_err()); // Comma
    assert!(validate_tool_name("tool@name").is_err()); // @ symbol
    assert!(validate_tool_name("tool#name").is_err()); // # symbol
    assert!(validate_tool_name("tool$name").is_err()); // $ symbol
    assert!(validate_tool_name("tool%name").is_err()); // % symbol
    assert!(validate_tool_name("tool&name").is_err()); // & symbol
    assert!(validate_tool_name("tool*name").is_err()); // * symbol
    assert!(validate_tool_name("tool+name").is_err()); // + symbol (not allowed in tool names)
    assert!(validate_tool_name("tool=name").is_err()); // = symbol
    assert!(validate_tool_name("tool?name").is_err()); // ? symbol
    assert!(validate_tool_name("tool!name").is_err()); // ! symbol
    assert!(validate_tool_name("tool[name").is_err()); // [ symbol
    assert!(validate_tool_name("tool]name").is_err()); // ] symbol
    assert!(validate_tool_name("tool{name").is_err()); // { symbol
    assert!(validate_tool_name("tool}name").is_err()); // } symbol
    assert!(validate_tool_name("tool|name").is_err()); // | symbol
    assert!(validate_tool_name("tool\\name").is_err()); // \ symbol
    assert!(validate_tool_name("tool/name").is_err()); // / symbol
    assert!(validate_tool_name("tool:name").is_err()); // : symbol
    assert!(validate_tool_name("tool;name").is_err()); // ; symbol
    assert!(validate_tool_name("tool\"name").is_err()); // " symbol
    assert!(validate_tool_name("tool'name").is_err()); // ' symbol
    assert!(validate_tool_name("tool<name").is_err()); // < symbol
    assert!(validate_tool_name("tool>name").is_err()); // > symbol
    assert!(validate_tool_name("tool\nname").is_err()); // Newline
    assert!(validate_tool_name("tool\tname").is_err()); // Tab
    assert!(validate_tool_name("tool\rname").is_err()); // Carriage return
}

#[test]
fn test_resource_uri_edge_cases() {
    // Valid edge cases
    assert!(validate_resource_uri("file:///path").is_ok());
    assert!(validate_resource_uri("http://example.com").is_ok());
    assert!(validate_resource_uri("https://example.com/path/to/resource").is_ok());
    assert!(validate_resource_uri("custom+scheme://path").is_ok());
    assert!(validate_resource_uri(&format!("file:///{}", "a".repeat(2040))).is_ok()); // Near max length

    // Invalid edge cases
    assert!(validate_resource_uri("").is_err()); // Empty
    assert!(validate_resource_uri(&format!("file:///{}", "a".repeat(2049))).is_err()); // Too long
    assert!(validate_resource_uri("invalid-uri").is_err()); // No scheme
    assert!(validate_resource_uri("://path").is_err()); // Empty scheme
    assert!(validate_resource_uri("file://").is_err()); // No path (but technically valid URI, so might pass)
    assert!(validate_resource_uri("file@://path").is_err()); // Invalid scheme character
    assert!(validate_resource_uri("file#://path").is_err()); // Invalid scheme character
    assert!(validate_resource_uri("file$://path").is_err()); // Invalid scheme character
}

#[test]
fn test_prompt_name_edge_cases() {
    // Valid edge cases
    assert!(validate_prompt_name("a").is_ok()); // Minimum length
    assert!(validate_prompt_name(&"a".repeat(128)).is_ok()); // Maximum length
    assert!(validate_prompt_name("a_b-c.d").is_ok()); // All allowed characters
    assert!(validate_prompt_name("A_B-C.D").is_ok()); // Uppercase
    assert!(validate_prompt_name("a1_b2-c3.d4").is_ok()); // With numbers

    // Invalid edge cases
    assert!(validate_prompt_name("").is_err()); // Empty
    assert!(validate_prompt_name(&"a".repeat(129)).is_err()); // Too long
    assert!(validate_prompt_name("prompt name").is_err()); // Space
    assert!(validate_prompt_name("prompt,name").is_err()); // Comma
    assert!(validate_prompt_name("prompt@name").is_err()); // @ symbol
    assert!(validate_prompt_name("prompt#name").is_err()); // # symbol
    assert!(validate_prompt_name("prompt$name").is_err()); // $ symbol
    assert!(validate_prompt_name("prompt%name").is_err()); // % symbol
    assert!(validate_prompt_name("prompt&name").is_err()); // & symbol
    assert!(validate_prompt_name("prompt*name").is_err()); // * symbol
    assert!(validate_prompt_name("prompt+name").is_err()); // + symbol
    assert!(validate_prompt_name("prompt=name").is_err()); // = symbol
    assert!(validate_prompt_name("prompt?name").is_err()); // ? symbol
    assert!(validate_prompt_name("prompt!name").is_err()); // ! symbol
    assert!(validate_prompt_name("prompt[name").is_err()); // [ symbol
    assert!(validate_prompt_name("prompt]name").is_err()); // ] symbol
    assert!(validate_prompt_name("prompt{name").is_err()); // { symbol
    assert!(validate_prompt_name("prompt}name").is_err()); // } symbol
    assert!(validate_prompt_name("prompt|name").is_err()); // | symbol
    assert!(validate_prompt_name("prompt\\name").is_err()); // \ symbol
    assert!(validate_prompt_name("prompt/name").is_err()); // / symbol
    assert!(validate_prompt_name("prompt:name").is_err()); // : symbol
    assert!(validate_prompt_name("prompt;name").is_err()); // ; symbol
    assert!(validate_prompt_name("prompt\"name").is_err()); // " symbol
    assert!(validate_prompt_name("prompt'name").is_err()); // ' symbol
    assert!(validate_prompt_name("prompt<name").is_err()); // < symbol
    assert!(validate_prompt_name("prompt>name").is_err()); // > symbol
    assert!(validate_prompt_name("prompt\nname").is_err()); // Newline
    assert!(validate_prompt_name("prompt\tname").is_err()); // Tab
    assert!(validate_prompt_name("prompt\rname").is_err()); // Carriage return
}

#[test]
fn test_unicode_edge_cases() {
    // Tool names should reject non-ASCII
    assert!(validate_tool_name("tëst").is_err()); // Non-ASCII character
    assert!(validate_tool_name("测试").is_err()); // Chinese characters
    assert!(validate_tool_name("тест").is_err()); // Cyrillic characters
    assert!(validate_tool_name("テスト").is_err()); // Japanese characters
    assert!(validate_tool_name("tëst").is_err()); // Accented character

    // Resource URIs can contain non-ASCII in path (URL encoded)
    // But scheme must be ASCII
    assert!(validate_resource_uri("file:///测试").is_ok()); // Non-ASCII in path is technically valid

    // Prompt names should reject non-ASCII
    assert!(validate_prompt_name("prömpt").is_err()); // Non-ASCII character
    assert!(validate_prompt_name("提示").is_err()); // Chinese characters
}
