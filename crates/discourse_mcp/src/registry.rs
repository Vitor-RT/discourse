use crate::primitive::{Kind, Primitive, Risk};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct Registry {
    primitives: Mutex<HashMap<(Kind, String), Primitive>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            primitives: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, primitive: Primitive) -> Result<Primitive, String> {
        let key = (primitive.kind, primitive.identifier.clone());
        let mut primitives = self.primitives.lock().unwrap();

        if primitives.contains_key(&key) {
            return Err(format!("MCP primitive already registered: {:?}", key));
        }

        primitives.insert(key, primitive.clone());
        Ok(primitive)
    }

    pub fn register_tool(
        &self,
        identifier: &str,
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
    ) -> Result<Primitive, String> {
        let primitive = Primitive::new(
            identifier,
            Kind::Tool,
            title,
            description,
            implementation,
            input_schema,
            output_schema,
            required_scopes,
            annotations,
            risk,
            provider,
            availability,
        )?;
        self.register(primitive)
    }

    pub fn register_resource(
        &self,
        identifier: &str,
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
    ) -> Result<Primitive, String> {
        let primitive = Primitive::new(
            identifier,
            Kind::Resource,
            title,
            description,
            implementation,
            input_schema,
            output_schema,
            required_scopes,
            annotations,
            risk,
            provider,
            availability,
        )?;
        self.register(primitive)
    }

    pub fn register_resource_template(
        &self,
        identifier: &str,
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
    ) -> Result<Primitive, String> {
        let primitive = Primitive::new(
            identifier,
            Kind::ResourceTemplate,
            title,
            description,
            implementation,
            input_schema,
            output_schema,
            required_scopes,
            annotations,
            risk,
            provider,
            availability,
        )?;
        self.register(primitive)
    }

    pub fn register_prompt(
        &self,
        identifier: &str,
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
    ) -> Result<Primitive, String> {
        let primitive = Primitive::new(
            identifier,
            Kind::Prompt,
            title,
            description,
            implementation,
            input_schema,
            output_schema,
            required_scopes,
            annotations,
            risk,
            provider,
            availability,
        )?;
        self.register(primitive)
    }

    pub fn find(&self, kind: Kind, identifier: &str) -> Option<Primitive> {
        let primitives = self.primitives.lock().unwrap();
        primitives.get(&(kind, identifier.to_string())).cloned()
    }

    pub fn all(&self, kind: Option<Kind>) -> Vec<Primitive> {
        let primitives = self.primitives.lock().unwrap();
        let mut values: Vec<Primitive> = primitives.values().cloned().collect();

        if let Some(k) = kind {
            values.retain(|p| p.kind == k);
        }

        values.sort_by(|a, b| a.identifier.cmp(&b.identifier));
        values
    }

    pub fn scopes(&self) -> Vec<String> {
        let primitives = self.primitives.lock().unwrap();
        let mut scopes: Vec<String> = primitives
            .values()
            .flat_map(|p| p.required_scopes.clone())
            .collect();
        scopes.sort();
        scopes.dedup();
        scopes
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
