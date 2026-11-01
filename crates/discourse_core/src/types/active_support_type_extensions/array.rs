#[derive(Debug, Clone, PartialEq)]
pub enum ArrayValue {
    Integer(i64),
    String(String),
}

pub struct ArrayTypeExtension {
    compact_blank: bool,
}

impl ArrayTypeExtension {
    pub fn new(compact_blank: bool) -> Self {
        Self { compact_blank }
    }

    pub fn serializable(&self) -> bool {
        false
    }

    pub fn cast_value(&self, value: &str) -> Vec<ArrayValue> {
        let mut result = Vec::new();

        let split_by = if value.contains(',') && value.contains('|') {
            // Ruby regex: `,(?!.*\|)|\|(?!.*,)`
            // Simplified approximation since Rust regex doesn't support lookahead easily without extra crate:
            // Just assume splitting by `,` or `|` if we need to. This mimics string splitting logic.
            // Actually, in Discourse, it normally splits by `,` or `|` whichever is prominent.
            // For now let's just split by `,` if present, else `|`.
            if value.matches(',').count() >= value.matches('|').count() {
                ","
            } else {
                "|"
            }
        } else if value.contains(',') {
            ","
        } else if value.contains('|') {
            "|"
        } else {
            // No delimiters, just the string itself.
            ""
        };

        let items: Vec<&str> = if split_by.is_empty() {
            vec![value]
        } else {
            value.split(split_by).collect()
        };

        for item in items {
            let item = item.trim();
            if self.compact_blank && item.is_empty() {
                continue;
            }
            if let Ok(num) = item.parse::<i64>() {
                result.push(ArrayValue::Integer(num));
            } else {
                result.push(ArrayValue::String(item.to_string()));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cast_value_string() {
        let ext = ArrayTypeExtension::new(true);
        assert_eq!(
            ext.cast_value("1, 2, 3"),
<<<<<<< HEAD
            vec![
                ArrayValue::Integer(1),
                ArrayValue::Integer(2),
                ArrayValue::Integer(3)
            ]
=======
            vec![ArrayValue::Integer(1), ArrayValue::Integer(2), ArrayValue::Integer(3)]
>>>>>>> origin/rust-migration-7847495834899956704
        );
        assert_eq!(
            ext.cast_value("a|b|c"),
            vec![
                ArrayValue::String("a".to_string()),
                ArrayValue::String("b".to_string()),
                ArrayValue::String("c".to_string())
            ]
        );
        assert_eq!(
            ext.cast_value("1, , 3"),
            vec![ArrayValue::Integer(1), ArrayValue::Integer(3)]
        );
    }
}
