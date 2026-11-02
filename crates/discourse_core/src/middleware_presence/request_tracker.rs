use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

// Constants translated from Ruby
const MAX_URL_LENGTH: usize = 8192;
const MAX_REFERRER_LENGTH: usize = 8192;
const MAX_SESSION_ID_LENGTH: usize = 32;
const MAX_USER_AGENT_LENGTH: usize = 8192;
const MAX_LANGUAGE_LENGTH: usize = 20;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TrackingData {
    pub track_view: bool,
    pub explicit_track_view: bool,
    pub deferred_track_view: bool,
    pub implicit_track_view: bool,
    pub browser_page_view: bool,
    pub is_beacon: bool,
    pub is_embed: bool,
    pub is_crawler: bool,
    pub has_auth_cookie: bool,
    pub topic_id: Option<i64>,
    pub tracking_url: Option<String>,
    pub tracking_referrer: Option<String>,
    pub tracking_session_id: Option<String>,
    pub tracking_language: Option<String>,
    pub user_agent: Option<String>,
    pub request_remote_ip: Option<String>,
    pub current_user_id: Option<i64>,
    pub current_username: Option<String>,
}

pub struct RequestTracker;

impl RequestTracker {
    pub fn is_beacon_tracking_request(method: &str, path: &str, beacon_path: &str) -> bool {
        method == "POST" && path == beacon_path
    }

    pub fn is_pageview_tracking_request(method: &str, path: &str, base_path: &str) -> bool {
        method == "POST" && path == format!("{}/pageview", base_path)
    }

    pub fn is_engagement_tracking_request(method: &str, path: &str, engagement_path: &str) -> bool {
        method == "POST" && path == engagement_path
    }

    pub fn extract_beacon_view_tracking_data(
        env: &HashMap<String, String>,
        body: Option<&Value>,
    ) -> TrackingData {
        let mut data = TrackingData {
            browser_page_view: true,
            is_beacon: true,
            ..Default::default()
        };

        if let Some(body_val) = body {
            if let Some(topic_id) = body_val.get("topic_id").and_then(|v| v.as_i64()) {
                data.topic_id = Some(topic_id);
            }
            if let Some(url) = body_val.get("url").and_then(|v| v.as_str()) {
                data.tracking_url = Some(url.chars().take(MAX_URL_LENGTH).collect());
            }
            if let Some(referrer) = body_val.get("referrer").and_then(|v| v.as_str()) {
                data.tracking_referrer = Some(referrer.chars().take(MAX_REFERRER_LENGTH).collect());
            }
            if let Some(session_id) = body_val.get("session_id").and_then(|v| v.as_str()) {
                data.tracking_session_id =
                    Some(session_id.chars().take(MAX_SESSION_ID_LENGTH).collect());
            }
            if let Some(language) = body_val.get("language").and_then(|v| v.as_str()) {
                data.tracking_language = Some(language.chars().take(MAX_LANGUAGE_LENGTH).collect());
            }
            if let Some(embed) = body_val.get("embed").and_then(|v| v.as_bool()) {
                data.is_embed = embed;
            }
        }

        if let Some(user_agent) = env.get("HTTP_USER_AGENT") {
            data.user_agent = Some(user_agent.chars().take(MAX_USER_AGENT_LENGTH).collect());
        }

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_beacon_tracking_request() {
        assert!(RequestTracker::is_beacon_tracking_request(
            "POST", "/beacon", "/beacon"
        ));
        assert!(!RequestTracker::is_beacon_tracking_request(
            "GET", "/beacon", "/beacon"
        ));
        assert!(!RequestTracker::is_beacon_tracking_request(
            "POST", "/other", "/beacon"
        ));
    }

    #[test]
    fn test_extract_beacon_view_tracking_data() {
        let mut env = HashMap::new();
        env.insert("HTTP_USER_AGENT".to_string(), "Mozilla/5.0".to_string());

        let body = json!({
            "topic_id": 123,
            "url": "http://example.com",
            "referrer": "http://ref.com",
            "session_id": "abc",
            "language": "en",
            "embed": true
        });

        let data = RequestTracker::extract_beacon_view_tracking_data(&env, Some(&body));

        assert_eq!(data.topic_id, Some(123));
        assert_eq!(data.tracking_url, Some("http://example.com".to_string()));
        assert_eq!(data.tracking_referrer, Some("http://ref.com".to_string()));
        assert_eq!(data.tracking_session_id, Some("abc".to_string()));
        assert_eq!(data.tracking_language, Some("en".to_string()));
        assert_eq!(data.is_embed, true);
        assert_eq!(data.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(data.is_beacon, true);
        assert_eq!(data.browser_page_view, true);
        assert_eq!(data.track_view, false);
    }
}
