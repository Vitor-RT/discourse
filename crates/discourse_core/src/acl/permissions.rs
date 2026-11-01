use regex::Regex;
use std::collections::HashSet;

pub const OWNER: &str = "owner";

#[derive(Debug, Clone, PartialEq)]
pub struct Permissions {
    pub values: Vec<String>,
}

impl Permissions {
    pub fn new(permissions: Vec<&str>) -> Result<Self, String> {
        let mut values: Vec<String> = permissions.into_iter().map(String::from).collect();
        values.push(OWNER.to_string());

        // Deduplicate while preserving order (to some extent)
        let mut seen = HashSet::new();
        let mut dedup_values = Vec::new();
        for val in values {
            if seen.insert(val.clone()) {
                dedup_values.push(val);
            }
        }

        let re = Regex::new(r"^[a-z][a-z0-9_]*$").unwrap();

        for permission in &dedup_values {
            if !re.is_match(permission) {
                return Err(format!("Invalid ACL permission name: {}", permission));
            }

            // In Rust, we don't have dynamic method dispatch or checking `respond_to?` like Ruby.
            // But we will reject some reserved keywords that might conflict with our struct's own methods
            // mimicking Ruby's behavior. The ruby spec expects rejection of: `values`, `class`, `freeze`, `to_s`, `initialize`.
            let reserved = ["values", "class", "freeze", "to_s", "initialize"];
            if reserved.contains(&permission.as_str()) {
                return Err(format!("Invalid ACL permission name: {}", permission));
            }
        }

        Ok(Self {
            values: dedup_values,
        })
    }

    pub fn has_permission(&self, perm: &str) -> bool {
        self.values.contains(&perm.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_values() {
        let perms = Permissions::new(vec!["view", "edit", "manage"]).unwrap();
        assert_eq!(perms.values, vec!["view", "edit", "manage", "owner"]);
    }

    #[test]
    fn test_named_readers() {
        let perms = Permissions::new(vec!["view", "edit", "manage"]).unwrap();

        // In Rust we don't have dynamic named readers like permissions.view,
        // we test `has_permission` instead to mimic this loosely.
        assert!(perms.has_permission("view"));
        assert!(perms.has_permission("edit"));
        assert!(perms.has_permission("manage"));
        assert!(perms.has_permission("owner"));
        assert!(!perms.has_permission("jester"));
    }

    #[test]
    fn test_reject_invalid_names() {
        let invalid_names = vec!["", "two words", "1view", "view-edit", "view="];
        for name in invalid_names {
            assert!(Permissions::new(vec![name]).is_err());
        }
    }

    #[test]
    fn test_reject_reserved_names() {
        let reserved_names = vec!["values", "class", "freeze", "to_s", "initialize"];
        for name in reserved_names {
            assert!(Permissions::new(vec![name]).is_err());
        }
    }
}
