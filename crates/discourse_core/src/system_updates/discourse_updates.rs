use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

//
// Traits and structures for interacting with the environment
//

pub trait RedisClient: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
    fn set_ex_nx(&self, key: &str, value: &str, ttl_seconds: u64) -> bool;
    fn del(&self, keys: &[&str]);
    fn lrange(&self, key: &str, start: isize, end: isize) -> Vec<String>;
    fn rpush(&self, key: &str, values: &[String]);
    fn hgetall(&self, key: &str) -> HashMap<String, String>;
    fn hget(&self, key: &str, field: &str) -> Option<String>;
    fn hset(&self, key: &str, field: &str, value: &str);
    fn mapped_hmset(&self, key: &str, fields: &HashMap<String, String>);
    fn keys(&self, pattern: &str) -> Vec<String>;
}

pub trait HttpClient: Send + Sync {
    fn get(&self, url: &str) -> Result<String, String>;
}

pub trait BackgroundJobs: Send + Sync {
    fn enqueue(&self, job_name: &str, args: &HashMap<String, String>);
}

#[derive(Clone)]
pub struct AppContext {
    pub redis: Arc<dyn RedisClient>,
    pub http: Arc<dyn HttpClient>,
    pub jobs: Arc<dyn BackgroundJobs>,
    pub git_version: String,
    pub git_branch: String,
    pub full_version: String,
    pub version_string: String,
    pub env: String,
    pub version_checks: bool,
    // Provide dependencies below
    // Settings can be abstracted if needed
}

//
// Core domain objects
//

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscourseVersionCheck {
    pub installed_version: String,
    pub installed_sha: Option<String>,
    pub installed_describe: String,
    pub git_branch: String,
    pub updated_at: Option<DateTime<Utc>>,
    pub latest_version: Option<String>,
    pub latest_pretty_version: Option<String>,
    pub latest_sha: Option<String>,
    pub missing_versions_count: Option<i32>,
    pub version_check_pending: bool,
    pub stale_data: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MissingVersion {
    pub version: String,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewFeature {
    #[serde(default)]
    pub discourse_version: Option<String>,
    #[serde(default)]
    pub plugin_name: Option<String>,
    pub created_at: String,
    // Other fields exist but these are used for filtering
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpcomingChangeFeedItem {
    pub title: String,
    pub description: String,
    pub link: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub released_at: DateTime<Utc>,
    pub screenshot_url: Option<String>,
    pub upcoming_change_setting_name: String,
}

//
// DiscourseUpdates module logic
//

pub struct DiscourseUpdates;

impl DiscourseUpdates {
    // Keys
    const LAST_INSTALLED_VERSION_KEY: &'static str = "last_installed_version";
    const LATEST_VERSION_KEY: &'static str = "discourse_latest_version";
    const LATEST_PRETTY_VERSION_KEY: &'static str = "discourse_latest_pretty_version";
    const LATEST_SHA_KEY: &'static str = "discourse_latest_sha";
    const MISSING_VERSIONS_COUNT_KEY: &'static str = "missing_versions_count";
    const UPDATED_AT_KEY: &'static str = "last_version_check_at";
    const MISSING_VERSIONS_LIST_KEY: &'static str = "missing_versions";
    const MISSING_VERSIONS_KEY_PREFIX: &'static str = "missing_version";
    const NEW_FEATURES_KEY: &'static str = "new_features";
    const LATEST_NEW_FEATURE_CREATED_AT_KEY: &'static str = "latest_new_feature_created_at";
    const REFRESH_LATEST_NEW_FEATURE_LOCK_KEY: &'static str = "refresh_latest_new_feature_lock";
    const LAST_VIEWED_FEATURE_DATES_FOR_USERS_KEY: &'static str =
        "last_viewed_feature_dates_for_users_hash";

    fn new_features_last_seen_key(user_id: i32) -> String {
        format!("new_features_last_seen_user_{}", user_id)
    }

    //
    // Accessors
    //

    pub fn last_installed_version(ctx: &AppContext) -> Option<String> {
        ctx.redis.get(Self::LAST_INSTALLED_VERSION_KEY)
    }

    pub fn set_last_installed_version(ctx: &AppContext, version: &str) {
        ctx.redis.set(Self::LAST_INSTALLED_VERSION_KEY, version);
    }

    pub fn latest_version(ctx: &AppContext) -> Option<String> {
        ctx.redis.get(Self::LATEST_VERSION_KEY)
    }

    pub fn set_latest_version(ctx: &AppContext, version: &str) {
        ctx.redis.set(Self::LATEST_VERSION_KEY, version);
    }

    pub fn latest_pretty_version(ctx: &AppContext) -> Option<String> {
        ctx.redis.get(Self::LATEST_PRETTY_VERSION_KEY)
    }

    pub fn set_latest_pretty_version(ctx: &AppContext, version: &str) {
        ctx.redis.set(Self::LATEST_PRETTY_VERSION_KEY, version);
    }

    pub fn latest_sha(ctx: &AppContext) -> Option<String> {
        ctx.redis.get(Self::LATEST_SHA_KEY)
    }

    pub fn set_latest_sha(ctx: &AppContext, sha: &str) {
        ctx.redis.set(Self::LATEST_SHA_KEY, sha);
    }

    pub fn missing_versions_count(ctx: &AppContext) -> Option<i32> {
        ctx.redis
            .get(Self::MISSING_VERSIONS_COUNT_KEY)
            .and_then(|s| s.parse().ok())
    }

    pub fn set_missing_versions_count(ctx: &AppContext, count: i32) {
        ctx.redis
            .set(Self::MISSING_VERSIONS_COUNT_KEY, &count.to_string());
    }

    pub fn updated_at(ctx: &AppContext) -> Option<DateTime<Utc>> {
        ctx.redis
            .get(Self::UPDATED_AT_KEY)
            .and_then(|s| s.parse().ok())
    }

    pub fn set_updated_at(ctx: &AppContext, time: DateTime<Utc>) {
        // Rust's default DateTime formatting is ISO 8601-like, matching as_json roughly
        ctx.redis
            .set(Self::UPDATED_AT_KEY, &format!("\"{}\"", time.to_rfc3339()));
    }

    pub fn current_version(ctx: &AppContext) -> String {
        Self::last_installed_version(ctx).unwrap_or_else(|| ctx.version_string.clone())
    }

    //
    // Missing Versions
    //

    pub fn missing_versions(ctx: &AppContext) -> Vec<HashMap<String, String>> {
        let keys = ctx.redis.lrange(Self::MISSING_VERSIONS_LIST_KEY, 0, 4);
        if keys.is_empty() {
            return vec![];
        }
        keys.into_iter()
            .map(|k| ctx.redis.hgetall(&k))
            .collect()
    }

    pub fn set_missing_versions(
        ctx: &AppContext,
        versions: Vec<MissingVersion>,
    ) -> Vec<MissingVersion> {
        let prev_keys = ctx.redis.lrange(Self::MISSING_VERSIONS_LIST_KEY, 0, 4);
        if !prev_keys.is_empty() {
            let mut keys_to_del: Vec<&str> = prev_keys.iter().map(|s| s.as_str()).collect();
            keys_to_del.push(Self::MISSING_VERSIONS_LIST_KEY);
            ctx.redis.del(&keys_to_del);
        }

        if !versions.is_empty() {
            let limit = versions.len().min(5);
            let mut version_keys = Vec::new();
            for v in versions.iter().take(limit) {
                let key = format!("{}:{}", Self::MISSING_VERSIONS_KEY_PREFIX, v.version);
                let mut hm = HashMap::new();
                hm.insert("version".to_string(), v.version.clone());
                hm.insert("notes".to_string(), v.notes.clone());
                ctx.redis.mapped_hmset(&key, &hm);
                version_keys.push(key);
            }
            ctx.redis
                .rpush(Self::MISSING_VERSIONS_LIST_KEY, &version_keys);
        }

        versions
    }

    //
    // Check Version
    //

    pub fn check_version(ctx: &AppContext) -> DiscourseVersionCheck {
        let installed_sha = if ctx.git_version == "unknown" {
            None
        } else {
            Some(ctx.git_version.clone())
        };

        let mut installed_describe = ctx.full_version.clone();
        // replace -commit_count with +commit_count
        let re = regex::Regex::new(r"-(\d+)-.*").unwrap();
        installed_describe = re
            .replace(&installed_describe, " +$1")
            .into_owned();

        let updated = Self::updated_at(ctx);

        let mut info = DiscourseVersionCheck {
            installed_version: ctx.version_string.clone(),
            installed_sha,
            installed_describe,
            git_branch: ctx.git_branch.clone(),
            updated_at: updated,
            latest_version: None,
            latest_pretty_version: None,
            latest_sha: None,
            missing_versions_count: None,
            version_check_pending: false,
            stale_data: false,
        };

        if updated.is_some() {
            info.latest_version = Self::latest_version(ctx);
            info.latest_pretty_version = Self::latest_pretty_version(ctx);
            info.latest_sha = Self::latest_sha(ctx);
            info.missing_versions_count = Self::missing_versions_count(ctx);
        }

        if ctx.version_checks {
            let mvc = info.missing_versions_count.unwrap_or(0);
            let latest = info.latest_version.as_deref().unwrap_or("");
            let installed = &info.installed_version;

            let is_stale_data = (mvc == 0 && latest != installed) || (mvc != 0 && latest == installed);

            let last_installed =
                Self::last_installed_version(ctx).unwrap_or_else(|| "".to_string());
            let never_checked = info.updated_at.is_none();
            let updated_since_last_check = last_installed != ctx.version_string;

            if never_checked || updated_since_last_check || is_stale_data {
                let mut args = HashMap::new();
                args.insert("all_sites".to_string(), "true".to_string());
                ctx.jobs.enqueue("call_discourse_hub", &args);
                info.version_check_pending = true;
                if info.updated_at.is_some() {
                    info.missing_versions_count = Some(0);
                }
            }

            let old_data = info
                .updated_at
                .map(|t| Utc::now() - t > Duration::hours(48))
                .unwrap_or(false);

            info.stale_data = info.version_check_pending || old_data || is_stale_data;
        }

        info
    }

    //
    // New Features
    //

    pub fn new_features_endpoint(ctx: &AppContext) -> String {
        if ctx.env == "production" {
            "https://meta.discourse.org/new-features.json".to_string()
        } else {
            "http://localhost:3000/new-features.json".to_string()
        }
    }

    pub fn new_features_endpoint_query_params() -> String {
        // Approximate 5 months ago end of month. For simplicity here we just use 150 days ago.
        let dt = Utc::now() - Duration::days(150);
        format!("?released_after={}", dt.format("%Y-%m-%d"))
    }

    pub fn new_features_full_endpoint_url(ctx: &AppContext) -> String {
        format!(
            "{}{}",
            Self::new_features_endpoint(ctx),
            Self::new_features_endpoint_query_params()
        )
    }

    pub fn new_features_response_json(ctx: &AppContext) -> Result<String, String> {
        ctx.http.get(&Self::new_features_full_endpoint_url(ctx))
    }

    pub fn update_new_features(ctx: &AppContext, response_json: Option<String>) {
        let json = response_json.unwrap_or_else(|| {
            Self::new_features_response_json(ctx).unwrap_or_else(|_| "[]".to_string())
        });
        ctx.redis.set(Self::NEW_FEATURES_KEY, &json);
        Self::refresh_latest_new_feature_created_at(ctx);
    }

    pub fn refresh_latest_new_feature_created_at(ctx: &AppContext) -> Option<DateTime<Utc>> {
        // In full Ruby implementation, this merges with UpcomingChanges and filters
        // For Rust translation, we parse new_features from redis.
        let raw = ctx.redis.get(Self::NEW_FEATURES_KEY)?;
        let mut features: Vec<NewFeature> = serde_json::from_str(&raw).unwrap_or_default();

        // Simulating the merge_new_features_with_upcoming_changes call since
        // the upcoming changes logic is separate.

        if features.is_empty() {
            Self::clear_latest_new_feature_created_at_cache(ctx);
            return None;
        }

        let max_entry = features.iter().max_by_key(|f| {
            DateTime::parse_from_rfc3339(&f.created_at)
                .map(|d| d.timestamp())
                .unwrap_or(0)
        })?;

        let max_dt = DateTime::parse_from_rfc3339(&max_entry.created_at)
            .map(|d| d.with_timezone(&Utc))
            .ok()?;

        ctx.redis.set(
            Self::LATEST_NEW_FEATURE_CREATED_AT_KEY,
            &max_dt.to_rfc3339(),
        );

        Some(max_dt)
    }

    pub fn clear_latest_new_feature_created_at_cache(ctx: &AppContext) {
        ctx.redis.del(&[Self::LATEST_NEW_FEATURE_CREATED_AT_KEY]);
    }

    pub fn enqueue_latest_new_feature_created_at_refresh(ctx: &AppContext) {
        if ctx.redis.set_ex_nx(Self::REFRESH_LATEST_NEW_FEATURE_LOCK_KEY, "1", 60) {
            ctx.jobs.enqueue("refresh_latest_new_feature", &HashMap::new());
        }
    }

    pub fn latest_new_feature_created_at(ctx: &AppContext) -> Option<DateTime<Utc>> {
        let cached = ctx.redis.get(Self::LATEST_NEW_FEATURE_CREATED_AT_KEY)?;
        DateTime::parse_from_rfc3339(&cached)
            .map(|d| d.with_timezone(&Utc))
            .ok()
    }

    pub fn has_unseen_features(ctx: &AppContext, user_id: i32) -> bool {
        let latest_ts = Self::latest_new_feature_created_at(ctx);
        if let Some(latest) = latest_ts {
            if let Some(last_seen) = Self::new_features_last_seen(ctx, user_id) {
                latest.timestamp() > last_seen.timestamp()
            } else {
                true
            }
        } else {
            Self::enqueue_latest_new_feature_created_at_refresh(ctx);
            false
        }
    }

    pub fn new_features_last_seen(ctx: &AppContext, user_id: i32) -> Option<DateTime<Utc>> {
        let last_seen = ctx.redis.get(&Self::new_features_last_seen_key(user_id))?;
        DateTime::parse_from_rfc3339(&last_seen)
            .map(|d| d.with_timezone(&Utc))
            .ok()
    }

    pub fn mark_new_features_as_seen(ctx: &AppContext, user_id: i32) {
        // Simplified version: relies on latest_new_feature_created_at.
        if let Some(dt) = Self::latest_new_feature_created_at(ctx) {
            ctx.redis.set(
                &Self::new_features_last_seen_key(user_id),
                &dt.to_rfc3339(),
            );
        }
    }

    pub fn get_last_viewed_feature_date(ctx: &AppContext, user_id: i32) -> Option<DateTime<Utc>> {
        let date_str = ctx
            .redis
            .hget(Self::LAST_VIEWED_FEATURE_DATES_FOR_USERS_KEY, &user_id.to_string())?;
        DateTime::parse_from_rfc3339(&date_str)
            .map(|d| d.with_timezone(&Utc))
            .ok()
    }

    pub fn bump_last_viewed_feature_date(
        ctx: &AppContext,
        user_id: i32,
        feature_date: DateTime<Utc>,
    ) {
        ctx.redis.hset(
            Self::LAST_VIEWED_FEATURE_DATES_FOR_USERS_KEY,
            &user_id.to_string(),
            &feature_date.to_rfc3339(),
        );
    }

    pub fn clean_state(ctx: &AppContext) {
        let mut keys_to_del = vec![
            Self::LAST_INSTALLED_VERSION_KEY.to_string(),
            Self::LATEST_VERSION_KEY.to_string(),
            Self::LATEST_PRETTY_VERSION_KEY.to_string(),
            Self::LATEST_SHA_KEY.to_string(),
            Self::MISSING_VERSIONS_COUNT_KEY.to_string(),
            Self::UPDATED_AT_KEY.to_string(),
            Self::MISSING_VERSIONS_LIST_KEY.to_string(),
            Self::NEW_FEATURES_KEY.to_string(),
            Self::LAST_VIEWED_FEATURE_DATES_FOR_USERS_KEY.to_string(),
            Self::LATEST_NEW_FEATURE_CREATED_AT_KEY.to_string(),
            Self::REFRESH_LATEST_NEW_FEATURE_LOCK_KEY.to_string(),
        ];

        let missing_keys = ctx.redis.keys(&format!("{}*", Self::MISSING_VERSIONS_KEY_PREFIX));
        keys_to_del.extend(missing_keys);

        let last_seen = ctx.redis.keys("new_features_last_seen_user_*");
        keys_to_del.extend(last_seen);

        let refs: Vec<&str> = keys_to_del.iter().map(|s| s.as_str()).collect();
        ctx.redis.del(&refs);
    }
}

//
// Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockRedis {
        store: Mutex<HashMap<String, String>>,
        lists: Mutex<HashMap<String, Vec<String>>>,
        hashes: Mutex<HashMap<String, HashMap<String, String>>>,
    }

    impl RedisClient for MockRedis {
        fn get(&self, key: &str) -> Option<String> {
            self.store.lock().unwrap().get(key).cloned()
        }
        fn set(&self, key: &str, value: &str) {
            self.store.lock().unwrap().insert(key.to_string(), value.to_string());
        }
        fn set_ex_nx(&self, key: &str, value: &str, _ttl: u64) -> bool {
            let mut s = self.store.lock().unwrap();
            if s.contains_key(key) {
                false
            } else {
                s.insert(key.to_string(), value.to_string());
                true
            }
        }
        fn del(&self, keys: &[&str]) {
            let mut s = self.store.lock().unwrap();
            for k in keys {
                s.remove(*k);
            }
        }
        fn lrange(&self, key: &str, start: isize, end: isize) -> Vec<String> {
            let l = self.lists.lock().unwrap();
            if let Some(list) = l.get(key) {
                let start = start.max(0) as usize;
                let end = if end < 0 {
                    list.len().saturating_sub(end.abs() as usize)
                } else {
                    (end as usize + 1).min(list.len())
                };
                if start < end {
                    list[start..end].to_vec()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }
        fn rpush(&self, key: &str, values: &[String]) {
            let mut l = self.lists.lock().unwrap();
            let entry = l.entry(key.to_string()).or_default();
            entry.extend(values.iter().cloned());
        }
        fn hgetall(&self, key: &str) -> HashMap<String, String> {
            let h = self.hashes.lock().unwrap();
            h.get(key).cloned().unwrap_or_default()
        }
        fn hget(&self, key: &str, field: &str) -> Option<String> {
            let h = self.hashes.lock().unwrap();
            h.get(key).and_then(|hm| hm.get(field)).cloned()
        }
        fn hset(&self, key: &str, field: &str, value: &str) {
            let mut h = self.hashes.lock().unwrap();
            let entry = h.entry(key.to_string()).or_default();
            entry.insert(field.to_string(), value.to_string());
        }
        fn mapped_hmset(&self, key: &str, fields: &HashMap<String, String>) {
            let mut h = self.hashes.lock().unwrap();
            let entry = h.entry(key.to_string()).or_default();
            entry.extend(fields.clone());
        }
        fn keys(&self, _pattern: &str) -> Vec<String> {
            vec![]
        }
    }

    struct MockJobs;
    impl BackgroundJobs for MockJobs {
        fn enqueue(&self, _job_name: &str, _args: &HashMap<String, String>) {}
    }

    struct MockHttp;
    impl HttpClient for MockHttp {
        fn get(&self, _url: &str) -> Result<String, String> {
            Ok("[]".to_string())
        }
    }

    fn setup_ctx() -> AppContext {
        AppContext {
            redis: Arc::new(MockRedis::default()),
            http: Arc::new(MockHttp),
            jobs: Arc::new(MockJobs),
            git_version: "abcdef".to_string(),
            git_branch: "main".to_string(),
            full_version: "3.2.0-beta1-123-gabcdef".to_string(),
            version_string: "3.2.0.beta1".to_string(),
            env: "development".to_string(),
            version_checks: true,
        }
    }

    #[test]
    fn test_set_and_get_latest_version() {
        let ctx = setup_ctx();
        DiscourseUpdates::set_latest_version(&ctx, "3.3.0");
        assert_eq!(DiscourseUpdates::latest_version(&ctx).unwrap(), "3.3.0");
    }

    #[test]
    fn test_missing_versions() {
        let ctx = setup_ctx();
        let versions = vec![
            MissingVersion {
                version: "3.2.0.beta2".to_string(),
                notes: "Notes 2".to_string(),
            },
            MissingVersion {
                version: "3.2.0.beta3".to_string(),
                notes: "Notes 3".to_string(),
            },
        ];
        DiscourseUpdates::set_missing_versions(&ctx, versions);

        let fetched = DiscourseUpdates::missing_versions(&ctx);
        assert_eq!(fetched.len(), 2);
        assert_eq!(fetched[0].get("version").unwrap(), "3.2.0.beta2");
    }

    #[test]
    fn test_check_version() {
        let ctx = setup_ctx();
        let info = DiscourseUpdates::check_version(&ctx);
        assert_eq!(info.installed_version, "3.2.0.beta1");
        assert_eq!(info.installed_sha.unwrap(), "abcdef");
        assert_eq!(info.installed_describe, "3.2.0-beta1 +123");
        assert!(info.version_check_pending); // Because never checked
    }

    #[test]
    fn test_new_features_last_seen() {
        let ctx = setup_ctx();
        let user_id = 42;
        assert_eq!(DiscourseUpdates::new_features_last_seen(&ctx, user_id), None);

        let now = Utc::now();
        ctx.redis.set(
            DiscourseUpdates::LATEST_NEW_FEATURE_CREATED_AT_KEY,
            &now.to_rfc3339()
        );

        DiscourseUpdates::mark_new_features_as_seen(&ctx, user_id);

        let seen = DiscourseUpdates::new_features_last_seen(&ctx, user_id).unwrap();
        assert_eq!(seen.timestamp(), now.timestamp());
    }
}
