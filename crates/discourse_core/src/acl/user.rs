use std::collections::{HashMap, HashSet};

use super::permissions;
use super::target::TargetAclItem;

pub trait AclUser {
    fn group_ids(&self) -> &[u64];
}

#[derive(Debug, Clone)]
pub struct User {
    pub target_lookup: HashMap<String, Vec<String>>,
    pub permission_lookup: HashMap<String, HashMap<String, Vec<u64>>>,
    pub groups: Vec<u64>,
}

impl AclUser for User {
    fn group_ids(&self) -> &[u64] {
        &self.groups
    }
}

pub struct TargetKey {
    pub target_type: String,
    pub target_id: u64,
}

impl User {
    pub fn new(flattened_acl_list: Vec<TargetAclItem>) -> Self {
        let mut target_lookup: HashMap<String, Vec<String>> = HashMap::new();
        let mut permission_lookup: HashMap<String, HashMap<String, Vec<u64>>> = HashMap::new();

        for acl in flattened_acl_list {
            let target_key = Self::target_key(&TargetKey {
                target_type: acl.target_type.clone(),
                target_id: acl.target_id,
            });

            target_lookup
                .entry(target_key)
                .or_default()
                .push(acl.permission.clone());

            let p_lookup = permission_lookup.entry(acl.permission.clone()).or_default();
            p_lookup
                .entry(acl.target_type.clone())
                .or_default()
                .push(acl.target_id);
        }

        Self {
            target_lookup,
            permission_lookup,
            groups: Vec::new(),
        }
    }

    pub fn target_key(target: &TargetKey) -> String {
        format!("{}_{}", target.target_type, target.target_id)
    }

    pub fn has_target_permission(&self, target: &TargetKey, permission: &str) -> bool {
        let key = Self::target_key(target);
        self.target_lookup
            .get(&key)
            .map(|perms| {
                perms.contains(&permission.to_string())
                    || perms.contains(&permissions::OWNER.to_string())
            })
            .unwrap_or(false)
    }

    pub fn has_any_target_permission(&self, target: &TargetKey, permissions: &[&str]) -> bool {
        let key = Self::target_key(target);
        self.target_lookup
            .get(&key)
            .map(|perms| {
                perms.contains(&permissions::OWNER.to_string())
                    || perms.iter().any(|p| permissions.contains(&p.as_str()))
            })
            .unwrap_or(false)
    }

    pub fn target_ids_with_permission(&self, target_class: &str, permission: &str) -> Vec<u64> {
        let target_ids = self
            .permission_lookup
            .get(permission)
            .and_then(|types| types.get(target_class).cloned())
            .unwrap_or_default();

        if permission == permissions::OWNER {
            return target_ids;
        }

        let owner_target_ids = self
            .permission_lookup
            .get(permissions::OWNER)
            .and_then(|types| types.get(target_class).cloned())
            .unwrap_or_default();

        let mut all_ids = HashSet::new();
        for id in target_ids {
            all_ids.insert(id);
        }
        for id in owner_target_ids {
            all_ids.insert(id);
        }

        let mut vec = all_ids.into_iter().collect::<Vec<_>>();
        vec.sort_unstable();
        vec
    }

    pub fn target_ids_with_any_permissions(
        &self,
        target_class: &str,
        permissions: &[&str],
    ) -> Vec<u64> {
        let mut all_ids = HashSet::new();
        for &perm in permissions {
            for id in self.target_ids_with_permission(target_class, perm) {
                all_ids.insert(id);
            }
        }
        let mut vec = all_ids.into_iter().collect::<Vec<_>>();
        vec.sort_unstable();
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_user() -> User {
        User::new(vec![
            TargetAclItem {
                acl_type: "group".to_string(),
                id: 10,
                permission: "view".to_string(),
                target_type: "Category".to_string(),
                target_id: 1,
            },
            TargetAclItem {
                acl_type: "group".to_string(),
                id: 10,
                permission: "edit".to_string(),
                target_type: "Category".to_string(),
                target_id: 1,
            },
            TargetAclItem {
                acl_type: "group".to_string(),
                id: 11,
                permission: "view".to_string(),
                target_type: "Topic".to_string(),
                target_id: 2,
            },
            TargetAclItem {
                acl_type: "user".to_string(),
                id: 12,
                permission: "owner".to_string(),
                target_type: "Topic".to_string(),
                target_id: 3,
            },
        ])
    }

    fn category() -> TargetKey {
        TargetKey {
            target_type: "Category".to_string(),
            target_id: 1,
        }
    }

    fn topic() -> TargetKey {
        TargetKey {
            target_type: "Topic".to_string(),
            target_id: 2,
        }
    }

    fn other_topic() -> TargetKey {
        TargetKey {
            target_type: "Topic".to_string(),
            target_id: 3,
        }
    }

    fn absent_topic() -> TargetKey {
        TargetKey {
            target_type: "Topic".to_string(),
            target_id: 4,
        }
    }

    #[test]
    fn test_has_target_permission() {
        let user = setup_user();

        assert!(user.has_target_permission(&category(), "view"));
        assert!(user.has_target_permission(&category(), "edit"));
        assert!(!user.has_target_permission(&topic(), "edit"));
        assert!(!user.has_target_permission(&absent_topic(), "view"));

        assert!(user.has_target_permission(&other_topic(), "view"));
        assert!(user.has_target_permission(&other_topic(), "manage"));
    }

    #[test]
    fn test_has_any_target_permission() {
        let user = setup_user();

        assert!(user.has_any_target_permission(&category(), &["manage", "edit"]));
        assert!(!user.has_any_target_permission(&topic(), &["edit", "manage"]));
        assert!(!user.has_any_target_permission(&absent_topic(), &["view", "edit"]));

        assert!(user.has_any_target_permission(&other_topic(), &["edit", "manage"]));
    }

    #[test]
    fn test_target_ids_with_permission() {
        let user = setup_user();

        assert_eq!(user.target_ids_with_permission("Category", "view"), vec![1]);
        assert_eq!(user.target_ids_with_permission("Topic", "view"), vec![2, 3]);
        assert_eq!(
            user.target_ids_with_permission("Category", "manage"),
            vec![] as Vec<u64>
        );
        assert_eq!(user.target_ids_with_permission("Topic", "edit"), vec![3]);
        assert_eq!(user.target_ids_with_permission("Topic", "owner"), vec![3]);
    }

    #[test]
    fn test_target_ids_with_any_permissions() {
        let user = setup_user();

        assert_eq!(
            user.target_ids_with_any_permissions("Category", &["view", "edit"]),
            vec![1]
        );
        assert_eq!(
            user.target_ids_with_any_permissions("Topic", &["view", "edit"]),
            vec![2, 3]
        );
    }
}
