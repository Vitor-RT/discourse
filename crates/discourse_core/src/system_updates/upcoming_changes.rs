use std::collections::HashMap;
use serde::{Deserialize, Serialize};

//
// Enums
//

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    Conceptual = -100,
    Experimental = 0,
    Alpha = 100,
    Beta = 200,
    Stable = 300,
    Permanent = 500,
    Never = 9999,
}

impl Status {
    pub fn value(&self) -> i32 {
        *self as i32
    }

    pub fn from_value(val: i32) -> Option<Status> {
        match val {
            -100 => Some(Status::Conceptual),
            0 => Some(Status::Experimental),
            100 => Some(Status::Alpha),
            200 => Some(Status::Beta),
            300 => Some(Status::Stable),
            500 => Some(Status::Permanent),
            9999 => Some(Status::Never),
            _ => None,
        }
    }

    pub fn from_str(s: &str) -> Option<Status> {
        match s {
            "conceptual" => Some(Status::Conceptual),
            "experimental" => Some(Status::Experimental),
            "alpha" => Some(Status::Alpha),
            "beta" => Some(Status::Beta),
            "stable" => Some(Status::Stable),
            "permanent" => Some(Status::Permanent),
            "never" => Some(Status::Never),
            _ => None,
        }
    }
}

//
// Traits for dependencies
//

pub trait SiteSettingsProvider: Send + Sync {
    fn humanized_name(&self, name: &str) -> String;
    fn description(&self, name: &str) -> String;
    fn plugins(&self) -> HashMap<String, String>;
    fn upcoming_change_metadata(&self) -> HashMap<String, HashMap<String, serde_json::Value>>;
    fn upcoming_change_site_settings(&self) -> Vec<String>;
    fn allow_enabled_for(&self, name: &str) -> Option<Vec<String>>;
    fn get_bool(&self, name: &str) -> bool;
}

pub trait CacheClient: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
    fn delete(&self, key: &str);
}

pub trait UserInfo: Send + Sync {
    fn upcoming_change_enabled(&self, change_name: &str) -> bool;
    fn belonging_to_group_ids(&self) -> Vec<i32>;
    fn is_system_user(&self) -> bool;
}

//
// Core Logic for UpcomingChanges
//

pub struct UpcomingChanges;

impl UpcomingChanges {
    pub fn previous_status(status: Status) -> Status {
        let val = status.value();
        let mut candidates = vec![
            Status::Conceptual,
            Status::Experimental,
            Status::Alpha,
            Status::Beta,
            Status::Stable,
            Status::Permanent,
            Status::Never,
        ];
        candidates.retain(|c| c.value() < val);
        candidates.into_iter().max_by_key(|c| c.value()).unwrap_or(Status::Conceptual)
    }

    pub fn next_status(status: Option<Status>) -> Option<Status> {
        let status = status?;
        let val = status.value();
        if val < Status::Experimental.value() || val >= Status::Stable.value() {
            return None;
        }

        let mut candidates = vec![
            Status::Conceptual,
            Status::Experimental,
            Status::Alpha,
            Status::Beta,
            Status::Stable,
            Status::Permanent,
            Status::Never,
        ];
        candidates.retain(|c| c.value() > val);
        candidates.into_iter().min_by_key(|c| c.value())
    }

    pub fn change_metadata(
        provider: &dyn SiteSettingsProvider,
        change_setting_name: &str,
    ) -> HashMap<String, serde_json::Value> {
        provider
            .upcoming_change_metadata()
            .get(change_setting_name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn change_status(
        provider: &dyn SiteSettingsProvider,
        change_setting_name: &str,
    ) -> Option<Status> {
        let metadata = Self::change_metadata(provider, change_setting_name);
        metadata
            .get("status")
            .and_then(|v| v.as_str())
            .and_then(Status::from_str)
    }

    pub fn change_status_value(
        provider: &dyn SiteSettingsProvider,
        change_setting_name: &str,
    ) -> Option<i32> {
        Self::change_status(provider, change_setting_name).map(|s| s.value())
    }

    pub fn not_yet_stable(provider: &dyn SiteSettingsProvider, change_setting_name: &str) -> bool {
        if let Some(val) = Self::change_status_value(provider, change_setting_name) {
            val < Status::Stable.value()
        } else {
            false
        }
    }

    pub fn stable_or_permanent(
        provider: &dyn SiteSettingsProvider,
        change_setting_name: &str,
    ) -> bool {
        if let Some(val) = Self::change_status_value(provider, change_setting_name) {
            val >= Status::Stable.value()
        } else {
            false
        }
    }

    pub fn meets_or_exceeds_status(
        provider: &dyn SiteSettingsProvider,
        change_setting_name: &str,
        status: Status,
    ) -> bool {
        if let Some(val) = Self::change_status_value(provider, change_setting_name) {
            val >= status.value()
        } else {
            false
        }
    }

    pub fn exists(provider: &dyn SiteSettingsProvider, change_setting_name: &str) -> bool {
        provider
            .upcoming_change_metadata()
            .contains_key(change_setting_name)
    }

    pub fn owning_plugin_configurable(
        provider: &dyn SiteSettingsProvider,
        change_setting_name: &str,
    ) -> bool {
        let plugins = provider.plugins();
        let plugin_name = plugins.get(change_setting_name);
        if plugin_name.is_none() {
            return true;
        }

        let md = Self::change_metadata(provider, change_setting_name);
        if let Some(req) = md.get("requires_plugin_enabled") {
            if req.as_bool() == Some(false) {
                return true;
            }
        }

        // This simulates Discourse.plugin_enabled? logic
        // Assuming plugin name ending in _enabled check on site settings
        let setting_key = format!("{}_enabled", plugin_name.unwrap());
        provider.get_bool(&setting_key)
    }

    pub fn settings_hidden_while_enabled(
        provider: &dyn SiteSettingsProvider,
        user: &dyn UserInfo,
    ) -> Vec<String> {
        let metadata = provider.upcoming_change_metadata();
        if metadata.is_empty() {
            return vec![];
        }

        let mut hidden = Vec::new();
        for (change_name, md) in metadata {
            if let Some(hs) = md.get("hide_settings") {
                if let Some(arr) = hs.as_array() {
                    // Check if enabled
                    if user.upcoming_change_enabled(&change_name) {
                        for item in arr {
                            if let Some(s) = item.as_str() {
                                hidden.push(s.to_string());
                            }
                        }
                    }
                }
            }
        }
        hidden
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSettings {
        metadata: HashMap<String, HashMap<String, serde_json::Value>>,
        bools: HashMap<String, bool>,
        plugins_map: HashMap<String, String>,
    }

    impl SiteSettingsProvider for MockSettings {
        fn humanized_name(&self, _name: &str) -> String { "".into() }
        fn description(&self, _name: &str) -> String { "".into() }
        fn plugins(&self) -> HashMap<String, String> { self.plugins_map.clone() }
        fn upcoming_change_metadata(&self) -> HashMap<String, HashMap<String, serde_json::Value>> {
            self.metadata.clone()
        }
        fn upcoming_change_site_settings(&self) -> Vec<String> { vec![] }
        fn allow_enabled_for(&self, _name: &str) -> Option<Vec<String>> { None }
        fn get_bool(&self, name: &str) -> bool {
            self.bools.get(name).copied().unwrap_or(false)
        }
    }

    struct MockUser {
        enabled: HashMap<String, bool>,
    }

    impl UserInfo for MockUser {
        fn upcoming_change_enabled(&self, name: &str) -> bool {
            self.enabled.get(name).copied().unwrap_or(false)
        }
        fn belonging_to_group_ids(&self) -> Vec<i32> { vec![] }
        fn is_system_user(&self) -> bool { false }
    }

    #[test]
    fn test_status_transitions() {
        assert_eq!(UpcomingChanges::next_status(Some(Status::Alpha)), Some(Status::Beta));
        assert_eq!(UpcomingChanges::next_status(Some(Status::Beta)), Some(Status::Stable));
        assert_eq!(UpcomingChanges::next_status(Some(Status::Stable)), None);
        assert_eq!(UpcomingChanges::previous_status(Status::Beta), Status::Alpha);
        assert_eq!(UpcomingChanges::previous_status(Status::Alpha), Status::Experimental);
    }

    #[test]
    fn test_hidden_settings() {
        let mut md = HashMap::new();
        let mut change = HashMap::new();
        change.insert("hide_settings".into(), serde_json::json!(["legacy_setting_1", "legacy_setting_2"]));
        md.insert("new_feature".into(), change);

        let provider = MockSettings {
            metadata: md,
            bools: HashMap::new(),
            plugins_map: HashMap::new(),
        };

        let mut enabled = HashMap::new();
        enabled.insert("new_feature".into(), true);
        let user = MockUser { enabled };

        let hidden = UpcomingChanges::settings_hidden_while_enabled(&provider, &user);
        assert_eq!(hidden.len(), 2);
        assert!(hidden.contains(&"legacy_setting_1".to_string()));
    }
}
