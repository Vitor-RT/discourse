use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum PresenceChannelError {
    NotFound,
    InvalidAccess,
    ConfigNotLoaded,
    InvalidConfig,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PresenceChannelState {
    pub message_bus_id: i64,
    pub user_ids: Option<Vec<i64>>,
    pub count: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PresenceChannelConfig {
    pub public: bool,
    pub count_only: bool,
    pub allowed_user_ids: Option<Vec<i64>>,
    pub allowed_group_ids: Option<Vec<i64>>,
}

pub struct PresenceChannel<'a> {
    pub name: &'a str,
    pub config: Option<PresenceChannelConfig>,
}

impl<'a> PresenceChannel<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name, config: None }
    }

    pub fn with_config(name: &'a str, config: PresenceChannelConfig) -> Self {
        Self {
            name,
            config: Some(config),
        }
    }

    pub fn redis_key_mutex(&self) -> String {
        format!("_presence_{}_mutex", self.name)
    }

    pub fn redis_key_zlist(&self) -> String {
        format!("_presence_{}_zlist", self.name)
    }

    pub fn redis_key_hash(&self) -> String {
        format!("_presence_{}_hash", self.name)
    }

    pub fn redis_key_config(&self) -> String {
        format!("_presence_{}_config", self.name)
    }

    pub fn redis_key_channel_list() -> &'static str {
        "_presence_channels"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_keys() {
        let channel = PresenceChannel::new("test_channel");

        assert_eq!(channel.redis_key_mutex(), "_presence_test_channel_mutex");
        assert_eq!(channel.redis_key_zlist(), "_presence_test_channel_zlist");
        assert_eq!(channel.redis_key_hash(), "_presence_test_channel_hash");
        assert_eq!(channel.redis_key_config(), "_presence_test_channel_config");
        assert_eq!(
            PresenceChannel::redis_key_channel_list(),
            "_presence_channels"
        );
    }

    #[test]
    fn test_with_config() {
        let config = PresenceChannelConfig {
            public: true,
            count_only: false,
            allowed_user_ids: Some(vec![1, 2, 3]),
            allowed_group_ids: None,
        };

        let channel = PresenceChannel::with_config("test_channel", config.clone());
        assert_eq!(channel.name, "test_channel");
        assert_eq!(channel.config, Some(config));
    }
}
