use serde_json::Value;
use serde_json::json;

pub struct OutputSchema;

impl OutputSchema {
    pub fn any() -> Value {
        json!({})
    }

    pub fn string() -> Value {
        json!({ "type": "string" })
    }

    pub fn integer() -> Value {
        json!({ "type": "integer" })
    }

    pub fn boolean() -> Value {
        json!({ "type": "boolean" })
    }

    pub fn string_or_null() -> Value {
        json!({ "type": ["string", "null"] })
    }

    pub fn integer_or_null() -> Value {
        json!({ "type": ["integer", "null"] })
    }

    pub fn boolean_or_null() -> Value {
        json!({ "type": ["boolean", "null"] })
    }

    pub fn array() -> Value {
        json!({ "type": "array" })
    }

    pub fn string_array() -> Value {
        json!({ "type": "array", "items": { "type": "string" } })
    }

    pub fn object_array() -> Value {
        json!({ "type": "array", "items": { "type": "object" } })
    }

    pub fn object_schema() -> Value {
        json!({ "type": "object" })
    }

    pub fn object(optional: &[&str], properties: serde_json::Map<String, Value>) -> Value {
        let mut required = vec![];
        for key in properties.keys() {
            if !optional.contains(&key.as_str()) {
                required.push(key.clone());
            }
        }

        json!({
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false,
        })
    }
}
