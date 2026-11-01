pub struct GithubAuthenticator;

impl GithubAuthenticator {
    pub fn name() -> &'static str {
        "github"
    }

    pub fn display_name() -> &'static str {
        "GitHub"
    }

    pub fn provider_url() -> &'static str {
        "https://github.com"
    }

    pub fn enable_setting() -> &'static str {
        "enable_github_logins"
    }

    pub fn required_settings() -> Vec<&'static str> {
        vec!["github_client_id", "github_client_secret"]
    }
}
