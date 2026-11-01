use std::collections::HashSet;

pub const RESOURCE_URL: &str = "resource_url";
pub const PROTECTED_RESOURCE_METADATA_URL: &str = "protected_resource_metadata_url";

#[derive(Debug, PartialEq)]
pub struct AuthenticationError(pub Option<String>);

#[derive(Clone, PartialEq)]
pub struct Client {
    pub id: i64,
}

#[derive(Clone, PartialEq)]
pub struct Authorization {
    pub id: i64,
    pub user_id: i64,
    pub client: Client,
    pub grant_version: i64,
    pub scopes: HashSet<String>,
    pub is_active: bool,
}

impl Authorization {
    pub fn active(&self) -> bool {
        self.is_active
    }
}

pub struct McpOauthAccessToken {
    pub token_hash: String,
    pub authorization: Authorization,
    pub client: Client,
    pub scopes: HashSet<String>,
    pub resource: String,
    pub grant_version: i64,
    pub user_id: i64,
}

impl McpOauthAccessToken {
    pub fn digest(token: &str) -> String {
        format!("hash_of_{}", token)
    }

    pub fn touch_last_used(&self) {
        // mock
    }
}

pub struct Request {
    pub authorization: Option<String>,
}

#[derive(Debug)]
pub struct RequestContext {
    pub token_hash: String,
}

impl RequestContext {
    pub fn new(token: &McpOauthAccessToken) -> Self {
        Self {
            token_hash: token.token_hash.clone(),
        }
    }
}

pub struct Challenge {
    pub status: String,
    pub header: String,
}

pub struct Authenticator<'a> {
    request: &'a Request,
}

impl<'a> Authenticator<'a> {
    pub fn new(request: &'a Request) -> Self {
        Self { request }
    }

    pub fn authenticate(
        &self,
        usable_tokens: &[McpOauthAccessToken],
        authorization_status_active: bool,
    ) -> Result<RequestContext, AuthenticationError> {
        let auth_header = match &self.request.authorization {
            Some(h) => h,
            None => return Err(AuthenticationError(None)),
        };

        let parts: Vec<&str> = auth_header.splitn(2, ' ').collect();
        if parts.is_empty() {
            return Err(AuthenticationError(None));
        }

        let scheme = parts[0];
        let raw_token = if parts.len() > 1 { parts[1] } else { "" };

        if scheme.is_empty() && raw_token.is_empty() {
            return Err(AuthenticationError(None));
        }

        if !scheme.eq_ignore_ascii_case("Bearer") || raw_token.is_empty() {
            return Err(AuthenticationError(Some("invalid_token".to_string())));
        }

        let token_hash = McpOauthAccessToken::digest(raw_token);
        let token = usable_tokens.iter().find(|t| t.token_hash == token_hash);

        let token = match token {
            Some(t) => t,
            None => return Err(AuthenticationError(Some("invalid_token".to_string()))),
        };

        let authorization = &token.authorization;
        let client = &token.client;

        let mut missing_scopes = false;
        for scope in &token.scopes {
            if !authorization.scopes.contains(scope) {
                missing_scopes = true;
                break;
            }
        }

        let valid = token.resource == RESOURCE_URL
            && authorization.active()
            && token.grant_version == authorization.grant_version
            && !missing_scopes
            && client == &authorization.client
            && token.user_id == authorization.user_id
            && authorization_status_active;

        if !valid {
            return Err(AuthenticationError(Some("invalid_token".to_string())));
        }

        token.touch_last_used();
        Ok(RequestContext::new(token))
    }

    pub fn challenge(scope: Option<&str>, error: Option<&str>) -> String {
        let mut values = vec![format!(
            "resource_metadata=\"{}\"",
            PROTECTED_RESOURCE_METADATA_URL
        )];
        if let Some(s) = scope {
            if !s.is_empty() {
                values.push(format!("scope=\"{}\"", s));
            }
        }
        if let Some(e) = error {
            if !e.is_empty() {
                values.push(format!("error=\"{}\"", e));
            }
        }
        format!("Bearer {}", values.join(", "))
    }
}
