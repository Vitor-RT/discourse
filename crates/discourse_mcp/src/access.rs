use crate::catalog::Policies;
use crate::primitive::Kind;
use crate::registry::Registry;
use std::collections::{HashMap, HashSet};

pub const INITIAL_SCOPE: &str = "initial_scope"; // Assuming a default value for test compilation

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct User {
    pub id: i64,
    pub is_admin: bool,
    pub is_active: bool,
    pub is_suspended: bool,
    pub is_staged: bool,
}

impl User {
    pub fn admin(&self) -> bool {
        self.is_admin
    }

    pub fn active(&self) -> bool {
        self.is_active
    }

    pub fn suspended(&self) -> bool {
        self.is_suspended
    }

    pub fn staged(&self) -> bool {
        self.is_staged
    }
}

pub struct SiteSetting {
    pub mcp_server_enabled: bool,
}

pub struct McpGroupScope {
    pub user_id: i64,
    pub scope: String,
}

impl McpGroupScope {
    pub fn ensure_admins() {
        // Mock implementation
    }
}

pub struct Access;

impl Access {
    pub fn eligible_scopes(
        user: Option<&User>,
        group_scopes: &[McpGroupScope],
        registry: &Registry,
    ) -> Vec<String> {
        if let Some(u) = user {
            let scopes_by_user =
                Self::eligible_scopes_by_user(vec![u.clone()], group_scopes, registry);
            scopes_by_user.get(&u.id).cloned().unwrap_or_default()
        } else {
            vec![]
        }
    }

    pub fn eligible_scopes_by_user(
        users: Vec<User>,
        group_scopes: &[McpGroupScope],
        registry: &Registry,
    ) -> HashMap<i64, Vec<String>> {
        let mut unique_users = HashMap::new();
        for user in users {
            unique_users.insert(user.id, user);
        }
        let users: Vec<User> = unique_users.into_values().collect();

        let mut scopes_by_user_id: HashMap<i64, Vec<String>> =
            users.iter().map(|u| (u.id, vec![])).collect();
        let eligible_users: Vec<&User> = users
            .iter()
            .filter(|u| Self::user_available(Some(*u)))
            .collect();

        if eligible_users.iter().any(|u| u.admin()) {
            McpGroupScope::ensure_admins();
        }

        if eligible_users.is_empty() {
            return scopes_by_user_id;
        }

        let eligible_user_ids: HashSet<i64> = eligible_users.iter().map(|u| u.id).collect();
        let registry_scopes: HashSet<String> = registry.scopes().into_iter().collect();

        for gs in group_scopes {
            if eligible_user_ids.contains(&gs.user_id) && registry_scopes.contains(&gs.scope) {
                if let Some(scopes) = scopes_by_user_id.get_mut(&gs.user_id) {
                    if !scopes.contains(&gs.scope) {
                        scopes.push(gs.scope.clone());
                    }
                }
            }
        }

        for scopes in scopes_by_user_id.values_mut() {
            if !scopes.is_empty() && !scopes.contains(&INITIAL_SCOPE.to_string()) {
                scopes.push(INITIAL_SCOPE.to_string());
            }
            scopes.sort();
        }

        scopes_by_user_id
    }

    pub fn user_available(user: Option<&User>) -> bool {
        match user {
            Some(u) => u.active() && !u.suspended() && !u.staged(),
            None => false,
        }
    }

    pub fn eligible_for_exposed_primitive(
        user: Option<&User>,
        site_setting: &SiteSetting,
        group_scopes: &[McpGroupScope],
        registry: &Registry,
        policies: &Policies,
        kinds_to_check: &[Kind], // Representing McpPrimitive.exposed.pluck(:kind, :identifier)
    ) -> bool {
        if !site_setting.mcp_server_enabled {
            return false;
        }

        let eligible_scopes: HashSet<String> = Self::eligible_scopes(user, group_scopes, registry)
            .into_iter()
            .collect();
        if eligible_scopes.is_empty() {
            return false;
        }

        for kind in kinds_to_check {
            let all_primitives = registry.all(Some(*kind));
            for primitive in all_primitives {
                if policies.exposed_identifiers.contains(&primitive.identifier) {
                    if primitive.available()
                        && primitive
                            .required_scopes
                            .iter()
                            .all(|scope| eligible_scopes.contains(scope))
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn allowed(
        user: Option<&User>,
        group_scopes: &[McpGroupScope],
        registry: &Registry,
    ) -> bool {
        !Self::eligible_scopes(user, group_scopes, registry).is_empty()
    }
}
