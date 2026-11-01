pub const DISCOURSE_MODULES: &[&str] = &[
    "deprecation-workflow",
    "lib/avatar-utils",
    "lib/case-converter",
    "lib/escape",
    "lib/get-url",
    "lib/object",
    "loader",
    "static/markdown-it/features",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_bundle_modules() {
        assert_eq!(DISCOURSE_MODULES.len(), 8);
        assert!(DISCOURSE_MODULES.contains(&"loader"));
    }
}
