pub struct FacebookAuthenticator;

impl FacebookAuthenticator {
    pub const AVATAR_SIZE: u32 = 480;

    pub fn name() -> &'static str {
        "facebook"
    }

    pub fn display_name() -> &'static str {
        "Facebook"
    }

    pub fn provider_url() -> &'static str {
        "https://www.facebook.com"
    }

    pub fn enable_setting() -> &'static str {
        "enable_facebook_logins"
    }

    pub fn required_settings() -> Vec<&'static str> {
        vec!["facebook_app_id", "facebook_app_secret"]
    }
}
