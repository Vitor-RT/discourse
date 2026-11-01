pub struct ParamEncoder;

impl ParamEncoder {
    pub fn encode(query: &str, args: &[&str]) -> String {
        let mut result = query.to_string();
        for arg in args {
            if let Some(pos) = result.find('?') {
                result.replace_range(pos..pos+1, arg);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(
            ParamEncoder::encode("SELECT * FROM users WHERE id = ?", &["1"]),
            "SELECT * FROM users WHERE id = 1"
        );
    }
}
