use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct AnonymousCacheHelper<'a> {
    pub env: &'a HashMap<String, String>,
    pub request_path: &'a str,
    pub request_method: &'a str,
    pub user_agent: Option<&'a str>,
}

impl<'a> AnonymousCacheHelper<'a> {
    pub fn new(
        env: &'a HashMap<String, String>,
        request_path: &'a str,
        request_method: &'a str,
        user_agent: Option<&'a str>,
    ) -> Self {
        Self {
            env,
            request_path,
            request_method,
            user_agent,
        }
    }

    pub fn get(&self) -> bool {
        self.request_method == "GET"
    }

    pub fn is_xhr(&self) -> bool {
        if let Some(requested_with) = self.env.get("HTTP_X_REQUESTED_WITH") {
            requested_with.eq_ignore_ascii_case("XMLHttpRequest")
        } else {
            false
        }
    }

    pub fn cache_key(&self) -> String {
        let xhr_flag = if self.is_xhr() { "t" } else { "f" };
        let accept = self
            .env
            .get("HTTP_ACCEPT")
            .unwrap_or(&"".to_string())
            .clone();
        let scheme = self
            .env
            .get("rack.url_scheme")
            .unwrap_or(&"".to_string())
            .clone();
        let host = self.env.get("HTTP_HOST").unwrap_or(&"".to_string()).clone();
        let uri = self
            .env
            .get("REQUEST_URI")
            .unwrap_or(&"".to_string())
            .clone();

        let mut key = format!(
            "ANON_CACHE_{}_{}_{}_{}{}",
            xhr_flag, accept, scheme, host, uri
        );

        if self.request_path.starts_with("/embed/") {
            key.push_str(&self.key_embed_referer());
        }

        key
    }

    pub fn key_embed_referer(&self) -> String {
        let default_referer = "".to_string();
        let referer = self.env.get("HTTP_REFERER").unwrap_or(&default_referer);
        let mut hasher = Sha256::new();
        hasher.update(referer);
        format!("|er={}", hex::encode(hasher.finalize()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_xhr() {
        let mut env = HashMap::new();
        env.insert(
            "HTTP_X_REQUESTED_WITH".to_string(),
            "XMLHttpRequest".to_string(),
        );
        let helper = AnonymousCacheHelper::new(&env, "/", "GET", None);
        assert!(helper.is_xhr());

        let mut env2 = HashMap::new();
        env2.insert(
            "HTTP_X_REQUESTED_WITH".to_string(),
            "xmlhttprequest".to_string(),
        );
        let helper2 = AnonymousCacheHelper::new(&env2, "/", "GET", None);
        assert!(helper2.is_xhr());

        let env3 = HashMap::new();
        let helper3 = AnonymousCacheHelper::new(&env3, "/", "GET", None);
        assert!(!helper3.is_xhr());
    }

    #[test]
    fn test_get() {
        let env = HashMap::new();
        let helper = AnonymousCacheHelper::new(&env, "/", "GET", None);
        assert!(helper.get());

        let helper_post = AnonymousCacheHelper::new(&env, "/", "POST", None);
        assert!(!helper_post.get());
    }
}
