use serde_json::Value;

pub struct ThemeSettingsValidator;

impl ThemeSettingsValidator {
    pub fn is_valid_value_type(value: &Value, type_id: u8) -> bool {
        match type_id {
            1 => value.is_i64(), // integer
            2 => value.is_i64() || value.is_f64(), // float
            3 => value.is_boolean(), // bool
            4 | 5 => value.is_string(), // list, icon
            6 => {
                // objects
                if let Some(arr) = value.as_array() {
                    arr.iter().all(|v| v.is_object())
                } else {
                    false
                }
            },
            _ => true, // default
        }
    }

    pub fn validate_value_in_range_int(value: i64, min: Option<i64>, max: Option<i64>) -> Vec<String> {
        let mut errors = Vec::new();
        if let Some(min_val) = min {
            if let Some(max_val) = max {
                if value < min_val || value > max_val {
                    errors.push(format!("value not valid min max {} {}", min_val, max_val));
                }
            } else if value < min_val {
                errors.push(format!("value not valid min {}", min_val));
            }
        } else if let Some(max_val) = max {
            if value > max_val {
                errors.push(format!("value not valid max {}", max_val));
            }
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_valid_value_type() {
        assert!(ThemeSettingsValidator::is_valid_value_type(&json!(5), 1));
        assert!(!ThemeSettingsValidator::is_valid_value_type(&json!("5"), 1));

        assert!(ThemeSettingsValidator::is_valid_value_type(&json!(5.5), 2));

        assert!(ThemeSettingsValidator::is_valid_value_type(&json!(true), 3));

        assert!(ThemeSettingsValidator::is_valid_value_type(&json!("string"), 4));

        assert!(ThemeSettingsValidator::is_valid_value_type(&json!([{"a": 1}]), 6));
        assert!(!ThemeSettingsValidator::is_valid_value_type(&json!(["a"]), 6));
    }

    #[test]
    fn test_validate_value_in_range_int() {
        let errs = ThemeSettingsValidator::validate_value_in_range_int(10, Some(1), Some(5));
        assert_eq!(errs.len(), 1);

        let errs2 = ThemeSettingsValidator::validate_value_in_range_int(10, Some(1), Some(15));
        assert_eq!(errs2.len(), 0);
    }
}
