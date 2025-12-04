//! Unit tests for schema generation.

use mcp_axum::schema::{extract_schema_from_docstring, schema_from_type};

#[test]
fn test_empty_docstring() {
    let schema = extract_schema_from_docstring("");
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"].as_object().unwrap().is_empty());
    assert!(schema["required"].as_array().unwrap().is_empty());
}

#[test]
fn test_no_arguments_section() {
    let docstring = "Just a description without arguments.";
    let schema = extract_schema_from_docstring(docstring);
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"].as_object().unwrap().is_empty());
}

#[test]
fn test_single_string_argument() {
    let docstring = r#"
Function description.

# Arguments
* `query` - Search query string (type: string)
"#;
    let schema = extract_schema_from_docstring(docstring);
    assert_eq!(schema["type"], "object");
    let props = schema["properties"].as_object().unwrap();
    assert!(props.contains_key("query"));
    assert_eq!(props["query"]["type"], "string");
    assert!(schema["required"].as_array().unwrap().contains(&"query".into()));
}

#[test]
fn test_multiple_arguments() {
    let docstring = r#"
Function description.

# Arguments
* `query` - Search query (type: string)
* `max_results` - Maximum results (type: integer, default: 10)
* `verbose` - Verbose output (type: boolean, default: false)
"#;
    let schema = extract_schema_from_docstring(docstring);
    let props = schema["properties"].as_object().unwrap();
    assert_eq!(props.len(), 3);
    assert_eq!(props["query"]["type"], "string");
    assert_eq!(props["max_results"]["type"], "integer");
    assert_eq!(props["max_results"]["default"], 10);
    assert_eq!(props["verbose"]["type"], "boolean");
    assert_eq!(props["verbose"]["default"], false);
    
    let required = schema["required"].as_array().unwrap();
    assert_eq!(required.len(), 1);
    assert!(required.contains(&"query".into()));
}

#[test]
fn test_numeric_defaults() {
    let docstring = r#"
# Arguments
* `count` - Count value (type: integer, default: 42)
* `ratio` - Ratio value (type: number, default: 3.14)
"#;
    let schema = extract_schema_from_docstring(docstring);
    let props = schema["properties"].as_object().unwrap();
    assert_eq!(props["count"]["default"], 42);
    assert_eq!(props["ratio"]["default"], 3.14);
}

#[test]
fn test_string_defaults() {
    let docstring = r#"
# Arguments
* `mode` - Mode setting (type: string, default: "normal")
* `prefix` - Prefix value (type: string, default: 'test')
"#;
    let schema = extract_schema_from_docstring(docstring);
    let props = schema["properties"].as_object().unwrap();
    assert_eq!(props["mode"]["default"], "normal");
    assert_eq!(props["prefix"]["default"], "test");
}

#[test]
fn test_boolean_defaults() {
    let docstring = r#"
# Arguments
* `enabled` - Enable flag (type: boolean, default: true)
* `disabled` - Disable flag (type: boolean, default: false)
"#;
    let schema = extract_schema_from_docstring(docstring);
    let props = schema["properties"].as_object().unwrap();
    assert_eq!(props["enabled"]["default"], true);
    assert_eq!(props["disabled"]["default"], false);
}

#[test]
fn test_type_variants() {
    let docstring = r#"
# Arguments
* `str1` - String type (type: String)
* `str2` - String type (type: &str)
* `num1` - Number type (type: f32)
* `num2` - Number type (type: f64)
* `int1` - Integer type (type: usize)
* `int2` - Integer type (type: u32)
* `int3` - Integer type (type: i64)
* `bool1` - Boolean type (type: bool)
"#;
    let schema = extract_schema_from_docstring(docstring);
    let props = schema["properties"].as_object().unwrap();
    assert_eq!(props["str1"]["type"], "string");
    assert_eq!(props["str2"]["type"], "string");
    assert_eq!(props["num1"]["type"], "number");
    assert_eq!(props["num2"]["type"], "number");
    assert_eq!(props["int1"]["type"], "integer");
    assert_eq!(props["int2"]["type"], "integer");
    assert_eq!(props["int3"]["type"], "integer");
    assert_eq!(props["bool1"]["type"], "boolean");
}

#[test]
fn test_description_cleaning() {
    let docstring = r#"
# Arguments
* `param` - This is a description (type: string, default: "value")
"#;
    let schema = extract_schema_from_docstring(docstring);
    let props = schema["properties"].as_object().unwrap();
    assert_eq!(props["param"]["description"], "This is a description");
}

#[test]
fn test_schema_from_type() {
    assert_eq!(schema_from_type("String")["type"], "string");
    assert_eq!(schema_from_type("&str")["type"], "string");
    assert_eq!(schema_from_type("usize")["type"], "integer");
    assert_eq!(schema_from_type("u32")["type"], "integer");
    assert_eq!(schema_from_type("i64")["type"], "integer");
    assert_eq!(schema_from_type("f32")["type"], "number");
    assert_eq!(schema_from_type("f64")["type"], "number");
    assert_eq!(schema_from_type("bool")["type"], "boolean");
    assert_eq!(schema_from_type("Unknown")["type"], "object");
}

#[test]
fn test_complex_docstring() {
    let docstring = r#"
Search for items with various filters.

This function searches for items matching the given criteria.

# Arguments
* `query` - Search query string (type: string)
* `limit` - Maximum number of results (type: integer, default: 10)
* `offset` - Number of results to skip (type: integer, default: 0)
* `include_metadata` - Include metadata in results (type: boolean, default: false)

# Returns
Returns a JSON array of matching items.
"#;
    let schema = extract_schema_from_docstring(docstring);
    let props = schema["properties"].as_object().unwrap();
    assert_eq!(props.len(), 4);
    assert!(schema["required"].as_array().unwrap().contains(&"query".into()));
}

#[test]
fn test_malformed_argument_line() {
    let docstring = r#"
# Arguments
* `valid` - Valid parameter (type: string)
* Not a valid line
* `another` - Another valid (type: integer)
"#;
    let schema = extract_schema_from_docstring(docstring);
    let props = schema["properties"].as_object().unwrap();
    assert_eq!(props.len(), 2);
    assert!(props.contains_key("valid"));
    assert!(props.contains_key("another"));
}

#[test]
fn test_no_type_specified() {
    let docstring = r#"
# Arguments
* `param` - Just a description without type
"#;
    let schema = extract_schema_from_docstring(docstring);
    let props = schema["properties"].as_object().unwrap();
    assert_eq!(props["param"]["type"], "object");
}

