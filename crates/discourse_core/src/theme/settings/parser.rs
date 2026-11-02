use serde_json::Value;

pub struct ThemeSettingsParser;

impl ThemeSettingsParser {
    pub fn extract_description(desc: &Value) -> Option<String> {
        if desc.is_string() {
            return Some(desc.as_str().unwrap().to_string());
        }
        if let Some(map) = desc.as_object() {
            if let Some(en) = map.get("en") {
                if let Some(s) = en.as_str() {
                    return Some(s.to_string());
                }
            }
        }
        None
    }

    pub fn create_opts(default: &Value, type_id: u8, raw_opts: &Value) -> Value {
        let mut opts = serde_json::Map::new();

        if let Some(desc) = raw_opts.get("description") {
            if let Some(extracted) = Self::extract_description(desc) {
                opts.insert("description".to_string(), Value::String(extracted));
            }
        }

        if type_id == 3 {
            let mut choices = Vec::new();
            if let Some(raw_choices) = raw_opts.get("choices").and_then(|c| c.as_array()) {
                choices = raw_choices.clone();
            }
            if !choices.contains(default) {
                choices.push(default.clone());
            }
            opts.insert("choices".to_string(), Value::Array(choices));
        }

        if type_id == 1 || type_id == 2 || type_id == 4 {
            if let Some(max) = raw_opts.get("max") {
                opts.insert("max".to_string(), max.clone());
            }
            if let Some(min) = raw_opts.get("min") {
                opts.insert("min".to_string(), min.clone());
            }
        }

        for field in ["textarea", "resolve_group_membership", "refresh"] {
            let val = raw_opts.get(field).and_then(|v| v.as_bool()).unwrap_or(false);
            opts.insert(field.to_string(), Value::Bool(val));
        }

        Value::Object(opts)
    }

    pub fn parse_yaml(yaml: &str) -> Result<Value, String> {
        serde_yaml::from_str(yaml).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_description_string() {
        let desc = json!("A description");
        assert_eq!(ThemeSettingsParser::extract_description(&desc).unwrap(), "A description");
    }

    #[test]
    fn test_extract_description_hash() {
        let desc = json!({ "en": "A description" });
        assert_eq!(ThemeSettingsParser::extract_description(&desc).unwrap(), "A description");
    }

    #[test]
    fn test_create_opts_enum() {
        let raw = json!({
            "choices": ["a", "b"]
        });
        let opts = ThemeSettingsParser::create_opts(&json!("c"), 3, &raw);
        let choices = opts.get("choices").unwrap().as_array().unwrap();

        assert_eq!(choices.len(), 3);
        assert!(choices.contains(&json!("a")));
        assert!(choices.contains(&json!("c")));
    }
}
