pub struct TwitterAuthenticator;

impl TwitterAuthenticator {
    pub fn name() -> &'static str {
        "twitter"
    }

    pub fn display_name() -> &'static str {
        "X / Twitter"
    }

    pub fn provider_url() -> &'static str {
        "https://x.com"
    }

    pub fn enable_setting() -> &'static str {
        "enable_twitter_logins"
    }

    pub fn required_settings() -> Vec<&'static str> {
        vec!["twitter_consumer_key", "twitter_consumer_secret"]
    }
}
