use std::collections::{HashMap, HashSet};

use super::permissions::{
    is_valid_permission_name, normalize_permission_name, normalize_permission_names,
};
use super::user::AclUser;

#[derive(Debug, Clone)]
pub struct TargetAclItem {
    pub acl_type: String,
    pub id: u64,
    pub permission: String,
    pub target_type: String,
    pub target_id: u64,
}

#[derive(Debug, Clone, Default)]
pub struct TargetAcl {
    pub group_permissions: HashMap<String, HashSet<u64>>,
    pub user_permissions: HashMap<String, HashSet<u64>>,
}

impl TargetAcl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn group_has_permission(&self, group: u64, permission: &str) -> bool {
        let perm = normalize_permission_name(permission);
        if !is_valid_permission_name(&perm) {
            return false;
        }

        if let Some(groups) = self.group_permissions.get("owner") {
            if groups.contains(&group) {
                return true;
            }
        }

        if let Some(groups) = self.group_permissions.get(&perm) {
            if groups.contains(&group) {
                return true;
            }
        }

        false
    }

    pub fn group_has_any_permission(&self, group: u64, permissions: &[&str]) -> bool {
        let perms = normalize_permission_names(permissions);

        if let Some(groups) = self.group_permissions.get("owner") {
            if groups.contains(&group) {
                return true;
            }
        }

        for perm in perms {
            if is_valid_permission_name(&perm) {
                if let Some(groups) = self.group_permissions.get(&perm) {
                    if groups.contains(&group) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn permission_group_ids(&self, permission: &str) -> Vec<u64> {
        let perm = normalize_permission_name(permission);
        if !is_valid_permission_name(&perm) {
            return Vec::new();
        }

        let mut group_ids = HashSet::new();

        if let Some(groups) = self.group_permissions.get("owner") {
            group_ids.extend(groups);
        }

        if let Some(groups) = self.group_permissions.get(&perm) {
            group_ids.extend(groups);
        }

        let mut result: Vec<u64> = group_ids.into_iter().collect();
        result.sort_unstable();
        result
    }

    pub fn group_ids_with_any_permission(&self, permissions: &[&str]) -> Vec<u64> {
        let perms = normalize_permission_names(permissions);
        let mut group_ids = HashSet::new();

        if let Some(groups) = self.group_permissions.get("owner") {
            group_ids.extend(groups);
        }

        for perm in perms {
            if is_valid_permission_name(&perm) {
                if let Some(groups) = self.group_permissions.get(&perm) {
                    group_ids.extend(groups);
                }
            }
        }

        let mut result: Vec<u64> = group_ids.into_iter().collect();
        result.sort_unstable();
        result
    }

    pub fn user_has_permission(&self, user_id: u64, permission: &str) -> bool {
        let perm = normalize_permission_name(permission);
        if !is_valid_permission_name(&perm) {
            return false;
        }

        if let Some(users) = self.user_permissions.get("owner") {
            if users.contains(&user_id) {
                return true;
            }
        }

        if let Some(users) = self.user_permissions.get(&perm) {
            if users.contains(&user_id) {
                return true;
            }
        }

        false
    }

    pub fn user_has_any_permission(&self, user_id: u64, permissions: &[&str]) -> bool {
        let perms = normalize_permission_names(permissions);

        if let Some(users) = self.user_permissions.get("owner") {
            if users.contains(&user_id) {
                return true;
            }
        }

        for perm in perms {
            if is_valid_permission_name(&perm) {
                if let Some(users) = self.user_permissions.get(&perm) {
                    if users.contains(&user_id) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn permission_user_ids(&self, permission: &str) -> Vec<u64> {
        let perm = normalize_permission_name(permission);
        if !is_valid_permission_name(&perm) {
            return Vec::new();
        }

        let mut user_ids = HashSet::new();

        if let Some(users) = self.user_permissions.get("owner") {
            user_ids.extend(users);
        }

        if let Some(users) = self.user_permissions.get(&perm) {
            user_ids.extend(users);
        }

        let mut result: Vec<u64> = user_ids.into_iter().collect();
        result.sort_unstable();
        result
    }

    pub fn user_ids_with_any_permission(&self, permissions: &[&str]) -> Vec<u64> {
        let perms = normalize_permission_names(permissions);
        let mut user_ids = HashSet::new();

        if let Some(users) = self.user_permissions.get("owner") {
            user_ids.extend(users);
        }

        for perm in perms {
            if is_valid_permission_name(&perm) {
                if let Some(users) = self.user_permissions.get(&perm) {
                    user_ids.extend(users);
                }
            }
        }

        let mut result: Vec<u64> = user_ids.into_iter().collect();
        result.sort_unstable();
        result
    }

    pub fn user_has_permission_via_group(&self, user: &dyn AclUser, permission: &str) -> bool {
        let group_ids = self.permission_group_ids(permission);
        user.group_ids().iter().any(|g| group_ids.contains(g))
    }

    pub fn user_has_any_permission_via_group(
        &self,
        user: &dyn AclUser,
        permissions: &[&str],
    ) -> bool {
        let group_ids = self.group_ids_with_any_permission(permissions);
        user.group_ids().iter().any(|g| group_ids.contains(g))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_acl() -> TargetAcl {
        let mut acl = TargetAcl::new();

        let mut view_groups = HashSet::new();
        view_groups.insert(1);
        acl.group_permissions.insert("view".to_string(), view_groups);

        let mut edit_groups = HashSet::new();
        edit_groups.insert(1);
        acl.group_permissions.insert("edit".to_string(), edit_groups);

        let mut owner_groups = HashSet::new();
        owner_groups.insert(22);
        acl.group_permissions.insert("owner".to_string(), owner_groups);

        let mut manage_users = HashSet::new();
        manage_users.insert(2);
        acl.user_permissions.insert("manage".to_string(), manage_users);

        let mut view_users = HashSet::new();
        view_users.insert(99);
        acl.user_permissions.insert("view".to_string(), view_users);

        let mut owner_users = HashSet::new();
        owner_users.insert(3);
        acl.user_permissions.insert("owner".to_string(), owner_users);

        acl
    }

    #[test]
    fn test_group_has_permission() {
        let acl = setup_acl();
        assert!(acl.group_has_permission(1, "view"));
        assert!(acl.group_has_permission(1, "edit"));
        assert!(!acl.group_has_permission(1, "manage"));
        assert!(!acl.group_has_permission(99, "view"));

        assert!(acl.group_has_permission(22, "view"));
        assert!(acl.group_has_permission(22, "manage"));
    }

    #[test]
    fn test_group_has_any_permission() {
        let acl = setup_acl();
        assert!(acl.group_has_any_permission(1, &["view", "manage"]));
        assert!(!acl.group_has_any_permission(99, &["view", "manage"]));

        assert!(acl.group_has_any_permission(22, &["manage", "own"]));
    }

    #[test]
    fn test_permission_group_ids() {
        let acl = setup_acl();
        let mut view_ids = acl.permission_group_ids("view");
        view_ids.sort();
        assert_eq!(view_ids, vec![1, 22]);

        assert_eq!(acl.permission_group_ids("edit"), vec![1, 22]);
        assert_eq!(acl.permission_group_ids("manage"), vec![22]);
    }

    #[test]
    fn test_group_ids_with_any_permission() {
        let acl = setup_acl();
        assert_eq!(
            acl.group_ids_with_any_permission(&["view", "edit"]),
            vec![1, 22]
        );
        assert_eq!(
            acl.group_ids_with_any_permission(&["view", "own"]),
            vec![1, 22]
        );
    }

    #[test]
    fn test_user_has_permission() {
        let acl = setup_acl();
        assert!(acl.user_has_permission(2, "manage"));
        assert!(!acl.user_has_permission(99, "manage"));
        assert!(!acl.user_has_permission(123456, "view"));
        assert!(!acl.user_has_permission(1, "edit"));

        assert!(acl.user_has_permission(3, "view"));
        assert!(acl.user_has_permission(3, "manage"));
    }

    #[test]
    fn test_user_has_any_permission() {
        let acl = setup_acl();
        assert!(acl.user_has_any_permission(2, &["view", "manage"]));
        assert!(!acl.user_has_any_permission(99, &["edit", "manage"]));
        assert!(!acl.user_has_any_permission(123456, &["view", "manage"]));

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
        assert_eq!(
            acl.user_ids_with_any_permission(&["view", "manage"]),
            vec![2, 3, 99]
        );
        assert_eq!(
            acl.user_ids_with_any_permission(&["view", "own"]),
            vec![3, 99]
        );
    }
}
