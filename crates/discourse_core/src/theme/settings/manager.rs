use serde_json::Value;

pub struct ThemeMock {
    pub id: i64,
}

pub struct ThemeSettingsManager {
    pub name: String,
    pub default_value: Value,
    pub theme: ThemeMock,
    pub type_id: u8,
}

impl ThemeSettingsManager {
    pub fn new(name: String, default: Value, theme: ThemeMock, type_id: u8) -> Self {
        ThemeSettingsManager {
            name,
            default_value: default,
            theme,
            type_id,
        }
    }

    pub fn value(&self) -> Value {
        self.default_value.clone()
    }
}

pub fn type_name_from_id(type_id: u8) -> &'static str {
    match type_id {
        1 => "integer",
        2 => "float",
        3 => "bool",
        4 => "string",
        5 => "list",
        6 => "enum",
        7 => "upload",
        8 => "icon",
        9 => "objects",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_theme_settings_manager_creation() {
        let manager = ThemeSettingsManager::new(
            "my_setting".into(),
            json!("default_val"),
            ThemeMock { id: 1 },
            4
        );

        assert_eq!(manager.name, "my_setting");
        assert_eq!(manager.value(), json!("default_val"));
        assert_eq!(manager.type_id, 4);
    }

    #[test]
    fn test_type_name_from_id() {
        assert_eq!(type_name_from_id(3), "bool");
        assert_eq!(type_name_from_id(9), "objects");
    }
}
