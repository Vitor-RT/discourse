use std::collections::{HashMap, HashSet};

use super::permissions;

#[derive(Debug, Clone, Default)]
pub struct PermissionIds {
    pub group_ids: Vec<u64>,
    pub user_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct TargetAclItem {
    pub acl_type: String, // "group" or "user"
    pub id: u64,
    pub permission: String,
    pub target_type: String,
    pub target_id: u64,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub group_lookup: HashMap<u64, Vec<String>>,
    pub user_lookup: HashMap<u64, Vec<String>>,
    pub permission_lookup: HashMap<String, PermissionIds>,
}

impl Target {
    pub fn new(flattened_acl_list: Vec<TargetAclItem>) -> Self {
        let mut group_lookup: HashMap<u64, Vec<String>> = HashMap::new();
        let mut user_lookup: HashMap<u64, Vec<String>> = HashMap::new();
        let mut permission_lookup: HashMap<String, PermissionIds> = HashMap::new();

        for acl in flattened_acl_list {
            let p_ids = permission_lookup.entry(acl.permission.clone()).or_default();

            if acl.acl_type == "group" {
                group_lookup
                    .entry(acl.id)
                    .or_default()
                    .push(acl.permission.clone());
                p_ids.group_ids.push(acl.id);
            }

            if acl.acl_type == "user" {
                user_lookup
                    .entry(acl.id)
                    .or_default()
                    .push(acl.permission.clone());
                p_ids.user_ids.push(acl.id);
            }
        }

        Self {
            group_lookup,
            user_lookup,
            permission_lookup,
        }
    }

    pub fn group_has_permission(&self, group_id: u64, permission: &str) -> bool {
        self.group_lookup
            .get(&group_id)
            .map(|perms| perms.contains(&permission.to_string()))
            .unwrap_or(false)
    }

    pub fn group_has_any_permission(&self, group_id: u64, permissions: &[&str]) -> bool {
        self.group_lookup
            .get(&group_id)
            .map(|perms| perms.iter().any(|p| permissions.contains(&p.as_str())))
            .unwrap_or(false)
    }

    pub fn permission_group_ids(&self, permission: &str) -> Vec<u64> {
        self.permission_lookup
            .get(permission)
            .map(|p_ids| p_ids.group_ids.clone())
            .unwrap_or_default()
    }

    pub fn group_ids_with_any_permission(&self, permissions: &[&str]) -> Vec<u64> {
        let mut ids = HashSet::new();
        for &perm in permissions {
            if let Some(p_ids) = self.permission_lookup.get(perm) {
                for &id in &p_ids.group_ids {
                    ids.insert(id);
                }
            }
        }
        let mut vec = ids.into_iter().collect::<Vec<_>>();
        vec.sort_unstable(); // For stable returns
        vec
    }

    pub fn user_has_permission(&self, user_id: u64, permission: &str) -> bool {
        self.user_lookup
            .get(&user_id)
            .map(|perms| {
                perms.contains(&permission.to_string())
                    || perms.contains(&permissions::OWNER.to_string())
            })
            .unwrap_or(false)
    }

    pub fn user_has_any_permission(&self, user_id: u64, perms: &[&str]) -> bool {
        self.user_lookup
            .get(&user_id)
            .map(|user_perms| {
                user_perms.contains(&permissions::OWNER.to_string())
                    || user_perms.iter().any(|p| perms.contains(&p.as_str()))
            })
            .unwrap_or(false)
    }

    pub fn permission_user_ids(&self, permission: &str) -> Vec<u64> {
        let user_ids = self
            .permission_lookup
            .get(permission)
            .map(|p_ids| p_ids.user_ids.clone())
            .unwrap_or_default();

        if permission == permissions::OWNER {
            return user_ids;
        }

        let owner_user_ids = self
            .permission_lookup
            .get(permissions::OWNER)
            .map(|p_ids| p_ids.user_ids.clone())
            .unwrap_or_default();

        let mut all_ids = HashSet::new();
        for id in user_ids {
            all_ids.insert(id);
        }
        for id in owner_user_ids {
            all_ids.insert(id);
        }

        let mut vec = all_ids.into_iter().collect::<Vec<_>>();
        vec.sort_unstable();
        vec
    }

    pub fn user_ids_with_any_permission(&self, perms: &[&str]) -> Vec<u64> {
        let mut all_ids = HashSet::new();
        for &perm in perms {
            for id in self.permission_user_ids(perm) {
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

    fn setup_acl() -> Target {
        Target::new(vec![
            TargetAclItem { acl_type: "group".to_string(), id: 1, permission: "view".to_string(), target_type: "Category".to_string(), target_id: 5 },
            TargetAclItem { acl_type: "group".to_string(), id: 1, permission: "edit".to_string(), target_type: "Category".to_string(), target_id: 5 },
            TargetAclItem { acl_type: "group".to_string(), id: 22, permission: "view".to_string(), target_type: "Category".to_string(), target_id: 5 },
            TargetAclItem { acl_type: "user".to_string(), id: 2, permission: "manage".to_string(), target_type: "Category".to_string(), target_id: 5 },
            TargetAclItem { acl_type: "user".to_string(), id: 99, permission: "view".to_string(), target_type: "Category".to_string(), target_id: 5 },
            TargetAclItem { acl_type: "user".to_string(), id: 3, permission: "owner".to_string(), target_type: "Category".to_string(), target_id: 5 },
        ])
    }

    #[test]
    fn test_group_has_permission() {
        let acl = setup_acl();
        assert!(acl.group_has_permission(1, "view"));
        assert!(acl.group_has_permission(1, "edit"));
        assert!(!acl.group_has_permission(22, "edit"));
        assert!(!acl.group_has_permission(123456, "view"));
        assert!(!acl.group_has_permission(99, "manage"));
    }

    #[test]
    fn test_group_has_any_permission() {
        let acl = setup_acl();
        assert!(acl.group_has_any_permission(1, &["manage", "edit"]));
        assert!(!acl.group_has_any_permission(22, &["edit", "manage"]));
        assert!(!acl.group_has_any_permission(123456, &["view", "edit"]));
    }

    #[test]
    fn test_permission_group_ids() {
        let acl = setup_acl();
        let mut view_ids = acl.permission_group_ids("view");
        view_ids.sort();
        assert_eq!(view_ids, vec![1, 22]);

        assert_eq!(acl.permission_group_ids("edit"), vec![1]);
        assert_eq!(acl.permission_group_ids("manage"), vec![] as Vec<u64>);
        assert_eq!(acl.permission_group_ids("own"), vec![] as Vec<u64>);
    }

    #[test]
    fn test_group_ids_with_any_permission() {
        let acl = setup_acl();
        assert_eq!(acl.group_ids_with_any_permission(&["view", "edit"]), vec![1, 22]);
        assert_eq!(acl.group_ids_with_any_permission(&["view", "own"]), vec![1, 22]);
    }

    #[test]
    fn test_user_has_permission() {
        let acl = setup_acl();
        assert!(acl.user_has_permission(2, "manage"));
        assert!(!acl.user_has_permission(99, "manage"));
        assert!(!acl.user_has_permission(123456, "view"));
        assert!(!acl.user_has_permission(1, "edit")); // non user entry

        // owner grants every permission
        assert!(acl.user_has_permission(3, "view"));
        assert!(acl.user_has_permission(3, "manage"));
    }

    #[test]
    fn test_user_has_any_permission() {
        let acl = setup_acl();
        assert!(acl.user_has_any_permission(2, &["view", "manage"]));
        assert!(!acl.user_has_any_permission(99, &["edit", "manage"]));
        assert!(!acl.user_has_any_permission(123456, &["view", "manage"]));

        // owner grants any requested permission
        assert!(acl.user_has_any_permission(3, &["edit", "manage"]));
    }

    #[test]
    fn test_permission_user_ids() {
        let acl = setup_acl();
        assert_eq!(acl.permission_user_ids("view"), vec![3, 99]);
        assert_eq!(acl.permission_user_ids("manage"), vec![2, 3]);
        assert_eq!(acl.permission_user_ids("edit"), vec![3]);
        assert_eq!(acl.permission_user_ids("own"), vec![3]);
        assert_eq!(acl.permission_user_ids("owner"), vec![3]);
    }

    #[test]
    fn test_user_ids_with_any_permission() {
        let acl = setup_acl();
        assert_eq!(acl.user_ids_with_any_permission(&["view", "manage"]), vec![2, 3, 99]);
        assert_eq!(acl.user_ids_with_any_permission(&["view", "own"]), vec![3, 99]);
    }
}
