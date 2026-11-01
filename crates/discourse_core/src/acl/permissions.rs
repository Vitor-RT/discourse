use regex::Regex;
use std::collections::HashSet;

pub const OWNER: &str = "owner";

pub fn is_valid_permission_name(name: &str) -> bool {
    let re = Regex::new(r"^[a-z][a-z0-9_]*$").unwrap();
    let reserved = ["values", "class", "freeze", "to_s", "initialize"];
    re.is_match(name) && !reserved.contains(&name)
}

pub fn normalize_permission_name(name: &str) -> String {
    name.trim().to_lowercase()
}

pub fn normalize_permission_names(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| normalize_permission_name(s)).collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct Permissions {
    pub values: Vec<String>,
}

impl Permissions {
    pub fn new(permissions: Vec<&str>) -> Result<Self, String> {
        let mut values: Vec<String> = permissions.into_iter().map(String::from).collect();
        values.push(OWNER.to_string());

        let mut seen = HashSet::new();
        let mut dedup_values = Vec::new();
        for val in values {
            if seen.insert(val.clone()) {
                dedup_values.push(val);
            }
        }

        for permission in &dedup_values {
            if !is_valid_permission_name(permission) {
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
