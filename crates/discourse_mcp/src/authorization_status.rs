use crate::access::{Access, McpGroupScope, SiteSetting, User};
use crate::primitive::Kind;
use crate::registry::Registry;
use std::collections::{HashMap, HashSet};

pub struct Policy {
    pub kind: Kind,
    pub identifier: String,
    pub consent_required_at: Option<i64>,
}

#[derive(Clone)]
pub struct Client {
    pub id: i64,
    pub is_blocked: bool,
    pub metadata_hash: String,
}

impl Client {
    pub fn blocked(&self) -> bool {
        self.is_blocked
    }
}

pub struct OAuthClientResolver;
impl OAuthClientResolver {
    pub fn allowed_by_current_policy(_client: &Client) -> bool {
        true // mock
    }
}

#[derive(Clone)]
pub struct Authorization {
    pub id: i64,
    pub user_id: i64,
    pub user: User,
    pub scopes: HashSet<String>,
    pub status: String,
    pub revoked_at: Option<i64>,
    pub client: Client,
    pub client_metadata_hash: String,
}

impl Authorization {
    pub fn consent_current(&self, _consent_required_at: Option<i64>) -> bool {
        true // mock
    }
}

pub struct AuthorizationStatus<'a> {
    authorization: &'a Authorization,
    eligible_scopes: &'a HashSet<String>,
    consent_required_at: Option<i64>,
    required_scopes: &'a HashSet<String>,
}

impl<'a> AuthorizationStatus<'a> {
    pub fn new(
        authorization: &'a Authorization,
        eligible_scopes: &'a HashSet<String>,
        consent_required_at: Option<i64>,
        required_scopes: &'a HashSet<String>,
    ) -> Self {
        Self {
            authorization,
            eligible_scopes,
            consent_required_at,
            required_scopes,
        }
    }

    pub fn for_authorizations(
        authorizations: &[Authorization],
        scopes_by_authorization_id: &HashMap<i64, HashSet<String>>,
        policies: &[Policy],
        registry: &Registry,
        group_scopes: &[McpGroupScope],
        site_setting: &SiteSetting,
    ) -> HashMap<i64, String> {
        if authorizations.is_empty() {
            return HashMap::new();
        }

        let users: Vec<User> = authorizations.iter().map(|a| a.user.clone()).collect();
        let eligible_scopes_by_user_id =
            Access::eligible_scopes_by_user(users, group_scopes, registry);
        let requirements = Self::consent_requirements(policies, registry);

        let mut statuses = HashMap::new();

        for authorization in authorizations {
            let default_scopes = &authorization.scopes;
            let required_scopes = scopes_by_authorization_id
                .get(&authorization.id)
                .unwrap_or(default_scopes);

            let default_eligible = vec![];
            let eligible_scopes_vec = eligible_scopes_by_user_id
                .get(&authorization.user_id)
                .unwrap_or(&default_eligible);
            let eligible_scopes: HashSet<String> = eligible_scopes_vec.iter().cloned().collect();

            let consent_required_at = Self::consent_required_at(&requirements, required_scopes);

            let status_obj = AuthorizationStatus::new(
                authorization,
                &eligible_scopes,
                consent_required_at,
                required_scopes,
            );

            statuses.insert(authorization.id, status_obj.status(site_setting));
        }

        statuses
    }

    pub fn consent_requirements(
        policies: &[Policy],
        registry: &Registry,
    ) -> Vec<(HashSet<String>, i64)> {
        policies
            .iter()
            .filter_map(|policy| {
                if let Some(required_at) = policy.consent_required_at {
                    if let Some(primitive) = registry.find(policy.kind, &policy.identifier) {
                        let scopes: HashSet<String> =
                            primitive.required_scopes.iter().cloned().collect();
                        return Some((scopes, required_at));
                    }
                }
                None
            })
            .collect()
    }

    pub fn consent_required_at(
        consent_requirements: &[(HashSet<String>, i64)],
        required_scopes: &HashSet<String>,
    ) -> Option<i64> {
        consent_requirements
            .iter()
            .filter_map(|(scopes, required_at)| {
                if scopes.is_empty() || scopes.difference(required_scopes).count() == 0 {
                    Some(*required_at)
                } else {
                    None
                }
            })
            .max()
    }

    pub fn status(&self, site_setting: &SiteSetting) -> String {
        if self.authorization.status == "revoked" || self.authorization.revoked_at.is_some() {
            return "revoked".to_string();
        }
        if self.authorization.status == "consent_required" {
            return "consent_required".to_string();
        }
        if !site_setting.mcp_server_enabled {
            return "server_disabled".to_string();
        }
        if !Access::user_available(Some(&self.authorization.user)) {
            return "user_unavailable".to_string();
        }
        if self.authorization.client.blocked() {
            return "client_blocked".to_string();
        }
        if !OAuthClientResolver::allowed_by_current_policy(&self.authorization.client) {
            return "client_not_approved".to_string();
        }
        if self.authorization.client_metadata_hash != self.authorization.client.metadata_hash
            || !self.authorization.consent_current(self.consent_required_at)
        {
            return "consent_required".to_string();
        }

        if self
            .required_scopes
            .difference(self.eligible_scopes)
            .count()
            > 0
        {
            return "access_removed".to_string();
        }

        "active".to_string()
    }
}
