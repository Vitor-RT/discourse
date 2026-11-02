use serde_json::Value;

pub trait GroupResolver {
    fn check_groups(&self, user_group_ids: &[i64]) -> Option<Value>;
}

// Simple base stub logic representing resolving
// normally this queries the database to match groups vs settings
// For core migration, we ensure the interface boundaries exist for FFI/usage.

pub struct BaseGroupResolver;

impl GroupResolver for BaseGroupResolver {
    fn check_groups(&self, _user_group_ids: &[i64]) -> Option<Value> {
        None
    }
}

pub struct ListSettingGroupResolver {
    pub value: String,
    pub delimiter: char,
}

impl ListSettingGroupResolver {
    pub fn parsed_groups(&self) -> Vec<String> {
        self.value
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

impl GroupResolver for ListSettingGroupResolver {
    fn check_groups(&self, _user_group_ids: &[i64]) -> Option<Value> {
        // Full db-backed resolution is stubbed; this validates basic structure translation
        Some(Value::String(self.value.clone()))
    }
}

pub struct ObjectSettingGroupResolver {
    pub value: Vec<Value>,
    pub schema: Value,
}

impl ObjectSettingGroupResolver {
    pub fn new(value: Vec<Value>, schema: Value) -> Self {
        ObjectSettingGroupResolver { value, schema }
    }
}

impl GroupResolver for ObjectSettingGroupResolver {
    fn check_groups(&self, _user_group_ids: &[i64]) -> Option<Value> {
        // Normally navigates through JSON schema looking for type="groups"
        // Stubbed implementation returning filtered value
        Some(Value::Array(self.value.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_base_resolver_stub() {
        let resolver = BaseGroupResolver;
        assert_eq!(resolver.check_groups(&[1, 2, 3]), None);
    }

    #[test]
    fn test_list_setting_parsed_groups() {
        let resolver = ListSettingGroupResolver {
            value: "a| b|c ".to_string(),
            delimiter: '|',
        };
        assert_eq!(resolver.parsed_groups(), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_object_setting_group_resolver() {
        let val = vec![json!({"a": 1})];
        let schema = json!({"properties": {"a": {"type": "integer"}}});
        let resolver = ObjectSettingGroupResolver::new(val, schema);
        assert_eq!(resolver.check_groups(&[]).unwrap(), json!([{"a": 1}]));
    }
}
