pub struct GoogleOauth2Authenticator;

impl GoogleOauth2Authenticator {
    pub const GROUPS_SCOPE: &'static str = "https://www.googleapis.com/auth/admin.directory.group.readonly";
    pub const GROUPS_DOMAIN: &'static str = "admin.googleapis.com";
    pub const GROUPS_PATH: &'static str = "/admin/directory/v1/groups";
    pub const OAUTH2_BASE_URL: &'static str = "https://oauth2.googleapis.com";

    pub fn name() -> &'static str {
        "google_oauth2"
    }

    pub fn display_name() -> &'static str {
        "Google"
    }

    pub fn provider_url() -> &'static str {
        "https://accounts.google.com"
    }

    pub fn enable_setting() -> &'static str {
        "enable_google_oauth2_logins"
    }

    pub fn required_settings() -> Vec<&'static str> {
        vec!["google_oauth2_client_id", "google_oauth2_client_secret"]
    }
}
