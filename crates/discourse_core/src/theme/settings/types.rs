use serde_json::Value;

pub trait ThemeSettingType {
    fn cast(value: &Value) -> Value;
}

pub struct BoolType;
impl ThemeSettingType for BoolType {
    fn cast(value: &Value) -> Value {
        match value {
            Value::Bool(b) => Value::Bool(*b),
            Value::String(s) => match s.to_lowercase().as_str() {
                "t" | "true" | "1" => Value::Bool(true),
                "f" | "false" | "0" | "" => Value::Bool(false),
                _ => Value::Bool(false),
            },
            Value::Number(n) => {
                if n.as_i64() == Some(1) {
                    Value::Bool(true)
                } else {
                    Value::Bool(false)
                }
            }
            Value::Null => Value::Bool(false),
            _ => Value::Bool(false),
        }
    }
}

pub struct EnumType;
impl ThemeSettingType for EnumType {
    fn cast(value: &Value) -> Value {
        match value {
            Value::String(s) => {
                if let Ok(n) = s.parse::<i64>() {
                    Value::Number(n.into())
                } else if let Ok(f) = s.parse::<f64>() {
                    Value::Number(serde_json::Number::from_f64(f).unwrap())
                } else {
                    Value::String(s.to_string())
                }
            }
            _ => value.clone(),
        }
    }
}

pub struct FloatType;
impl ThemeSettingType for FloatType {
    fn cast(value: &Value) -> Value {
        match value {
            Value::Number(n) => Value::Number(n.clone()),
            Value::String(s) => {
                if let Ok(f) = s.parse::<f64>() {
                    Value::Number(serde_json::Number::from_f64(f).unwrap())
                } else {
                    Value::Null
                }
            }
            _ => Value::Null,
        }
    }
}

pub struct IntegerType;
impl ThemeSettingType for IntegerType {
    fn cast(value: &Value) -> Value {
        match value {
            Value::Number(n) => {
                if n.is_i64() {
                    Value::Number(n.clone())
                } else {
                    Value::Null
                }
            }
            Value::String(s) => {
                if let Ok(i) = s.parse::<i64>() {
                    Value::Number(i.into())
                } else {
                    Value::Null
                }
            }
            _ => Value::Null,
        }
    }
}

pub struct StringType;
impl ThemeSettingType for StringType {
    fn cast(value: &Value) -> Value {
        match value {
            Value::String(s) => Value::String(s.clone()),
            Value::Null => Value::String(String::new()),
            _ => Value::String(value.to_string()),
        }
    }
}

pub struct ListType;
impl ThemeSettingType for ListType {
    fn cast(value: &Value) -> Value {
        match value {
            Value::String(s) => Value::String(s.clone()),
            Value::Array(a) => {
                let joined: Vec<String> = a
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                Value::String(joined.join("|"))
            }
            _ => Value::String(String::new()),
        }
    }
}

pub struct ObjectsType;
impl ThemeSettingType for ObjectsType {
    fn cast(value: &Value) -> Value {
        match value {
            Value::Array(a) => {
                let mut out = Vec::new();
                for item in a {
                    if item.is_object() {
                        out.push(item.clone());
                    }
                }
                Value::Array(out)
            }
            Value::String(s) => {
                if let Ok(parsed) = serde_json::from_str::<Value>(s) {
                    if parsed.is_array() {
                        return ObjectsType::cast(&parsed);
                    }
                }
                Value::Array(Vec::new())
            }
            _ => Value::Array(Vec::new()),
        }
    }
}

// Icon and Upload normally reference other specific behavior (like stripping or file ID resolution),
// but their base values are effectively String/Integer in DB representation.
pub struct IconType;
impl ThemeSettingType for IconType {
    fn cast(value: &Value) -> Value {
        StringType::cast(value)
    }
}

pub struct UploadType;
impl ThemeSettingType for UploadType {
    fn cast(value: &Value) -> Value {
        IntegerType::cast(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_bool_type_cast() {
        assert_eq!(BoolType::cast(&json!(true)), json!(true));
        assert_eq!(BoolType::cast(&json!("true")), json!(true));
        assert_eq!(BoolType::cast(&json!("1")), json!(true));
        assert_eq!(BoolType::cast(&json!(1)), json!(true));
        assert_eq!(BoolType::cast(&json!("false")), json!(false));
        assert_eq!(BoolType::cast(&json!("")), json!(false));
        assert_eq!(BoolType::cast(&json!(null)), json!(false));
    }

    #[test]
    fn test_enum_type_cast() {
        assert_eq!(EnumType::cast(&json!("10")), json!(10));
        assert_eq!(EnumType::cast(&json!("10.5")), json!(10.5));
        assert_eq!(EnumType::cast(&json!("hello")), json!("hello"));
        assert_eq!(EnumType::cast(&json!(true)), json!(true));
    }

    #[test]
    fn test_float_type_cast() {
        assert_eq!(FloatType::cast(&json!(10.5)), json!(10.5));
        assert_eq!(FloatType::cast(&json!("10.5")), json!(10.5));
        assert_eq!(FloatType::cast(&json!("abc")), json!(null));
    }

    #[test]
    fn test_integer_type_cast() {
        assert_eq!(IntegerType::cast(&json!(10)), json!(10));
        assert_eq!(IntegerType::cast(&json!("10")), json!(10));
        assert_eq!(IntegerType::cast(&json!("10.5")), json!(null));
        assert_eq!(IntegerType::cast(&json!("abc")), json!(null));
    }

    #[test]
    fn test_string_type_cast() {
        assert_eq!(StringType::cast(&json!("hello")), json!("hello"));
        assert_eq!(StringType::cast(&json!(null)), json!(""));
        assert_eq!(StringType::cast(&json!(123)), json!("123"));
    }

    #[test]
    fn test_list_type_cast() {
        assert_eq!(ListType::cast(&json!("a|b")), json!("a|b"));
        assert_eq!(ListType::cast(&json!(["a", "b"])), json!("a|b"));
        assert_eq!(ListType::cast(&json!(123)), json!(""));
    }

    #[test]
    fn test_objects_type_cast() {
        assert_eq!(
            ObjectsType::cast(&json!([{"a": 1}, {"b": 2}])),
            json!([{"a": 1}, {"b": 2}])
        );
        assert_eq!(
            ObjectsType::cast(&json!([{"a": 1}, "string"])),
            json!([{"a": 1}])
        ); // Drops non-objects
        assert_eq!(ObjectsType::cast(&json!("[{\"a\": 1}]")), json!([{"a": 1}]));
        assert_eq!(ObjectsType::cast(&json!("invalid")), json!([]));
    }
}
