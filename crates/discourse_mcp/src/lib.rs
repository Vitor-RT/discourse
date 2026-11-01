pub mod output_schema;
pub mod primitive;
pub mod registry;
pub mod catalog;
pub mod access;
pub mod authenticator;
pub mod authorization_status;
pub mod core_tools;

#[cfg(test)]
mod tests {
    use super::output_schema::OutputSchema;
    use super::primitive::{Primitive, Kind, Risk};
    use std::collections::HashMap;
    use serde_json::json;

    #[test]
    fn test_output_schema() {
        assert_eq!(OutputSchema::any(), json!({}));
        assert_eq!(OutputSchema::string(), json!({ "type": "string" }));
    }

    #[test]
    fn test_primitive_valid() {
        let primitive = Primitive::new(
            "test_tool",
            Kind::Tool,
            "Test Tool",
            "A test tool",
            "implementation",
            Some(json!({"type": "object"})),
            None,
            vec![],
            HashMap::new(),
            Risk::Read,
            "core",
            None
        ).unwrap();

        assert_eq!(primitive.identifier, "test_tool");
    }

    #[test]
    fn test_primitive_invalid_identifier() {
        let err = Primitive::new(
            "invalid identifier with spaces!",
            Kind::Tool,
            "Test Tool",
            "A test tool",
            "implementation",
            Some(json!({"type": "object"})),
            None,
            vec![],
            HashMap::new(),
            Risk::Read,
            "core",
            None
        ).unwrap_err();

        assert_eq!(err, "invalid MCP primitive identifier");
    }
}
