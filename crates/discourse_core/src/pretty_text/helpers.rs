pub fn sanitize_cooked(cooked: &str) -> String {
    cooked.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_cooked() {
        let input = "<p>Hello</p>";
        assert_eq!(sanitize_cooked(input), "<p>Hello</p>");
    }
}
