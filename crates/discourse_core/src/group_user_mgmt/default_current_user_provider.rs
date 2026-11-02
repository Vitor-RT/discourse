use std::collections::HashMap;

pub const CURRENT_USER_KEY: &str = "_DISCOURSE_CURRENT_USER";
pub const USER_TOKEN_KEY: &str = "_DISCOURSE_USER_TOKEN";
pub const HEADER_API_KEY: &str = "HTTP_API_KEY";
pub const USER_API_KEY: &str = "HTTP_USER_API_KEY";
pub const API_KEY_ENV: &str = "_DISCOURSE_API";
pub const USER_API_KEY_ENV: &str = "_DISCOURSE_USER_API";
pub const BAD_TOKEN: &str = "_DISCOURSE_BAD_TOKEN";
pub const USER_API_CLIENT_ID: &str = "HTTP_USER_API_CLIENT_ID";

pub trait Env {
    fn get(&self, key: &str) -> Option<&String>;
    fn insert(&mut self, key: String, value: String);
    fn delete(&mut self, key: &str);
}

impl Env for HashMap<String, String> {
    fn get(&self, key: &str) -> Option<&String> {
        HashMap::get(self, key)
    }
    fn insert(&mut self, key: String, value: String) {
        HashMap::insert(self, key, value);
    }
    fn delete(&mut self, key: &str) {
        HashMap::remove(self, key);
    }
}

pub trait AuthDatabase {
    fn get_user_from_shared_key(&self, shared_key: &str) -> Option<i32>;
    fn lookup_auth_token(&self, auth_token: &str, ip: &str, user_agent: &str, path: &str) -> Option<i32>;
    fn is_suspended_or_inactive(&self, user_id: i32) -> bool;
    fn lookup_api_user(&self, api_key: &str, env: &dyn Env) -> Option<i32>;
    fn lookup_user_api_key(&self, hashed_key: &str) -> Option<i32>;
    fn ensure_allowed_user_api_key(&self, user_id: i32, env: &dyn Env) -> bool;
    fn update_last_used_user_api_key(&self, user_id: i32, client_id: &str);
}

pub struct DefaultCurrentUserProvider<'a, D: AuthDatabase, E: Env> {
    env: &'a mut E,
    db: &'a D,
    request_ip: String,
    request_path: String,
    user_agent: String,
    auth_token: Option<String>,
}

impl<'a, D: AuthDatabase, E: Env> DefaultCurrentUserProvider<'a, D, E> {
    pub fn new(env: &'a mut E, db: &'a D, request_ip: String, request_path: String, user_agent: String, auth_token: Option<String>) -> Self {
        Self { env, db, request_ip, request_path, user_agent, auth_token }
    }

    pub fn current_user(&mut self) -> Result<Option<i32>, String> {
        if let Some(user_id_str) = self.env.get(CURRENT_USER_KEY) {
            return Ok(user_id_str.parse().ok());
        }

        if let Some(shared_key) = self.env.get("HTTP_X_SHARED_SESSION_KEY") {
            let mut user = self.db.get_user_from_shared_key(shared_key);
            if let Some(uid) = user {
                if self.db.is_suspended_or_inactive(uid) {
                    user = None;
                }
            }
            if let Some(uid) = user {
                self.env.insert(CURRENT_USER_KEY.to_string(), uid.to_string());
            }
            return Ok(user);
        }

        let mut current_user = None;

        if let Some(token) = &self.auth_token {
            current_user = self.db.lookup_auth_token(token, &self.request_ip, &self.user_agent, &self.request_path);
            if current_user.is_none() {
                self.env.insert(BAD_TOKEN.to_string(), "true".to_string());
                return Err("Invalid Access".to_string());
            }
        } else if self.env.get("HTTP_DISCOURSE_LOGGED_IN").is_some() {
            self.env.insert(BAD_TOKEN.to_string(), "true".to_string());
        }

        if let Some(api_key) = self.env.get(HEADER_API_KEY).cloned() {
            current_user = self.db.lookup_api_user(&api_key, self.env);
            if current_user.is_none() {
                return Err("invalid_api_credentials".to_string());
            }
            if let Some(uid) = current_user {
                if self.db.is_suspended_or_inactive(uid) {
                    return Err("Invalid Access".to_string());
                }
            }
            self.env.insert(API_KEY_ENV.to_string(), "true".to_string());
        }

        if let Some(user_api_key) = self.env.get(USER_API_KEY).cloned() {
            let hashed = format!("hash_{}", user_api_key); // Mocking hash_key logic
            let user_api_key_obj_user_id = self.db.lookup_user_api_key(&hashed);

            if user_api_key_obj_user_id.is_none() {
                return Err("Invalid Access".to_string());
            }

            let uid = user_api_key_obj_user_id.unwrap();

            if !self.db.ensure_allowed_user_api_key(uid, self.env) {
                return Err("Invalid Access".to_string());
            }

            if self.db.is_suspended_or_inactive(uid) {
                return Err("Invalid Access".to_string());
            }

            if let Some(client_id) = self.env.get(USER_API_CLIENT_ID).cloned() {
                self.db.update_last_used_user_api_key(uid, &client_id);
            }

            current_user = Some(uid);
            self.env.insert(USER_API_KEY_ENV.to_string(), "true".to_string());
        }

        if let Some(uid) = current_user {
            if self.db.is_suspended_or_inactive(uid) {
                current_user = None;
            }
        }

        if let Some(uid) = current_user {
            self.env.insert(CURRENT_USER_KEY.to_string(), uid.to_string());
        }

        Ok(current_user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDb {
        user_from_shared_key: Option<i32>,
        user_from_auth_token: Option<i32>,
        user_from_api_key: Option<i32>,
        user_from_user_api_key: Option<i32>,
        is_suspended: bool,
        allowed_user_api_key: bool,
    }

    impl AuthDatabase for MockDb {
        fn get_user_from_shared_key(&self, _shared_key: &str) -> Option<i32> { self.user_from_shared_key }
        fn lookup_auth_token(&self, _auth_token: &str, _ip: &str, _user_agent: &str, _path: &str) -> Option<i32> { self.user_from_auth_token }
        fn is_suspended_or_inactive(&self, _user_id: i32) -> bool { self.is_suspended }
        fn lookup_api_user(&self, _api_key: &str, _env: &dyn Env) -> Option<i32> { self.user_from_api_key }
        fn lookup_user_api_key(&self, _hashed_key: &str) -> Option<i32> { self.user_from_user_api_key }
        fn ensure_allowed_user_api_key(&self, _user_id: i32, _env: &dyn Env) -> bool { self.allowed_user_api_key }
        fn update_last_used_user_api_key(&self, _user_id: i32, _client_id: &str) {}
    }

    #[test]
    fn test_current_user_already_set() {
        let mut env = HashMap::new();
        env.insert(CURRENT_USER_KEY.to_string(), "1".to_string());
        let db = MockDb { user_from_shared_key: None, user_from_auth_token: None, user_from_api_key: None, user_from_user_api_key: None, is_suspended: false, allowed_user_api_key: false };
        let mut provider = DefaultCurrentUserProvider::new(&mut env, &db, "".to_string(), "".to_string(), "".to_string(), None);

        assert_eq!(provider.current_user().unwrap(), Some(1));
    }

    #[test]
    fn test_shared_session_key() {
        let mut env = HashMap::new();
        env.insert("HTTP_X_SHARED_SESSION_KEY".to_string(), "session123".to_string());
        let db = MockDb { user_from_shared_key: Some(2), user_from_auth_token: None, user_from_api_key: None, user_from_user_api_key: None, is_suspended: false, allowed_user_api_key: false };
        let mut provider = DefaultCurrentUserProvider::new(&mut env, &db, "".to_string(), "".to_string(), "".to_string(), None);

        assert_eq!(provider.current_user().unwrap(), Some(2));
        assert_eq!(provider.env.get(CURRENT_USER_KEY).unwrap(), "2");
    }

    #[test]
    fn test_shared_session_key_suspended() {
        let mut env = HashMap::new();
        env.insert("HTTP_X_SHARED_SESSION_KEY".to_string(), "session123".to_string());
        let db = MockDb { user_from_shared_key: Some(2), user_from_auth_token: None, user_from_api_key: None, user_from_user_api_key: None, is_suspended: true, allowed_user_api_key: false };
        let mut provider = DefaultCurrentUserProvider::new(&mut env, &db, "".to_string(), "".to_string(), "".to_string(), None);

        assert_eq!(provider.current_user().unwrap(), None); // user is None because suspended
    }

    #[test]
    fn test_auth_token_success() {
        let mut env = HashMap::new();
        let db = MockDb { user_from_shared_key: None, user_from_auth_token: Some(3), user_from_api_key: None, user_from_user_api_key: None, is_suspended: false, allowed_user_api_key: false };
        let mut provider = DefaultCurrentUserProvider::new(&mut env, &db, "127.0.0.1".to_string(), "/".to_string(), "Browser".to_string(), Some("token123".to_string()));

        assert_eq!(provider.current_user().unwrap(), Some(3));
        assert_eq!(provider.env.get(CURRENT_USER_KEY).unwrap(), "3");
    }

    #[test]
    fn test_auth_token_failure() {
        let mut env = HashMap::new();
        let db = MockDb { user_from_shared_key: None, user_from_auth_token: None, user_from_api_key: None, user_from_user_api_key: None, is_suspended: false, allowed_user_api_key: false };
        let mut provider = DefaultCurrentUserProvider::new(&mut env, &db, "127.0.0.1".to_string(), "/".to_string(), "Browser".to_string(), Some("bad_token".to_string()));

        let res = provider.current_user();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Invalid Access");
        assert_eq!(provider.env.get(BAD_TOKEN).unwrap(), "true");
    }

    #[test]
    fn test_api_key_success() {
        let mut env = HashMap::new();
        env.insert(HEADER_API_KEY.to_string(), "api_key_value".to_string());
        let db = MockDb { user_from_shared_key: None, user_from_auth_token: None, user_from_api_key: Some(4), user_from_user_api_key: None, is_suspended: false, allowed_user_api_key: false };
        let mut provider = DefaultCurrentUserProvider::new(&mut env, &db, "".to_string(), "".to_string(), "".to_string(), None);

        assert_eq!(provider.current_user().unwrap(), Some(4));
        assert_eq!(provider.env.get(API_KEY_ENV).unwrap(), "true");
    }

    #[test]
    fn test_api_key_invalid() {
        let mut env = HashMap::new();
        env.insert(HEADER_API_KEY.to_string(), "api_key_value".to_string());
        let db = MockDb { user_from_shared_key: None, user_from_auth_token: None, user_from_api_key: None, user_from_user_api_key: None, is_suspended: false, allowed_user_api_key: false };
        let mut provider = DefaultCurrentUserProvider::new(&mut env, &db, "".to_string(), "".to_string(), "".to_string(), None);

        let res = provider.current_user();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "invalid_api_credentials");
    }

    #[test]
    fn test_user_api_key_success() {
        let mut env = HashMap::new();
        env.insert(USER_API_KEY.to_string(), "user_api_key_value".to_string());
        let db = MockDb { user_from_shared_key: None, user_from_auth_token: None, user_from_api_key: None, user_from_user_api_key: Some(5), is_suspended: false, allowed_user_api_key: true };
        let mut provider = DefaultCurrentUserProvider::new(&mut env, &db, "".to_string(), "".to_string(), "".to_string(), None);

        assert_eq!(provider.current_user().unwrap(), Some(5));
        assert_eq!(provider.env.get(USER_API_KEY_ENV).unwrap(), "true");
    }
}
