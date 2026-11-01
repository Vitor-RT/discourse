use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Tool,
    Resource,
    ResourceTemplate,
    Prompt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Risk {
    Read,
    Write,
    Destructive,
    Moderation,
    Administration,
    ExternalSideEffect,
}

#[derive(Debug, Clone)]
pub struct Primitive {
    pub identifier: String,
    pub kind: Kind,
    pub title: String,
    pub description: String,
    pub implementation: String,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
    pub required_scopes: Vec<String>,
    pub annotations: HashMap<String, Value>,
    pub risk: Risk,
    pub provider: String,
    pub availability: Option<fn() -> bool>,
}

impl Primitive {
    pub fn new(
        identifier: &str,
        kind: Kind,
        title: &str,
        description: &str,
        implementation: &str,
        input_schema: Option<Value>,
        output_schema: Option<Value>,
        required_scopes: Vec<String>,
        annotations: HashMap<String, Value>,
        risk: Risk,
        provider: &str,
        availability: Option<fn() -> bool>,
    ) -> Result<Self, String> {
        let input_schema = input_schema.unwrap_or_else(
            || serde_json::json!({ "type": "object", "additionalProperties": false }),
        );

        let re_identifier = Regex::new(r"^[A-Za-z0-9_.-]{1,128}$").unwrap();
        if !re_identifier.is_match(identifier) {
            return Err("invalid MCP primitive identifier".to_string());
        }

        if title.trim().is_empty() {
            return Err("MCP primitive title is required".to_string());
        }

        if description.trim().is_empty() {
            return Err("MCP primitive description is required".to_string());
        }

        if implementation.trim().is_empty() {
            return Err("MCP primitive implementation is required".to_string());
        }

        if kind == Kind::Tool {
            if let Some(obj) = input_schema.as_object() {
                if obj.get("type").and_then(|v| v.as_str()) != Some("object") {
                    return Err("MCP tool input schema must describe an object".to_string());
                }
            } else {
                return Err("MCP tool input schema must describe an object".to_string());
            }
        }

        let re_scope = Regex::new(r"^[a-z0-9][a-z0-9:_-]{0,127}$").unwrap();
        for scope in &required_scopes {
            if !re_scope.is_match(scope) {
                return Err("invalid MCP primitive scope".to_string());
            }
        }

        let allowed_annotations = vec![
            "title",
            "readOnlyHint",
            "destructiveHint",
            "idempotentHint",
            "openWorldHint",
        ];

        for key in annotations.keys() {
            if !allowed_annotations.contains(&key.as_str()) {
                return Err("invalid MCP primitive annotations".to_string());
            }
        }
        for (k, v) in &annotations {
            if k != "title" && !v.is_boolean() {
                return Err("invalid MCP primitive annotations".to_string());
            }
        }

        Ok(Self {
            identifier: identifier.to_string(),
            kind,
            title: title.to_string(),
            description: description.to_string(),
            implementation: implementation.to_string(),
            input_schema,
            output_schema,
            required_scopes,
            annotations,
            risk,
            provider: provider.to_string(),
            availability,
        })
    }

    pub fn available(&self) -> bool {
        match self.availability {
            Some(f) => f(),
            None => true,
        }
    }

    pub fn consent_relevant(&self) -> bool {
        matches!(
            self.risk,
            Risk::Destructive | Risk::Moderation | Risk::Administration | Risk::ExternalSideEffect
        )
    }
}
