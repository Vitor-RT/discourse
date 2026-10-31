pub struct SymbolTypeExtension;

impl SymbolTypeExtension {
    pub fn new() -> Self {
        Self
    }

    pub fn serializable(&self) -> bool {
        false
    }

    pub fn cast_value(&self, value: Option<&str>) -> Option<String> {
        match value {
            Some(v) if !v.trim().is_empty() => Some(v.trim().to_string()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cast_value() {
        let ext = SymbolTypeExtension::new();
        assert_eq!(ext.cast_value(None), None);
        assert_eq!(ext.cast_value(Some("")), None);
        assert_eq!(ext.cast_value(Some("  ")), None);
        assert_eq!(ext.cast_value(Some("test")), Some("test".to_string()));
        assert_eq!(ext.cast_value(Some(" test ")), Some("test".to_string()));
    }
}
