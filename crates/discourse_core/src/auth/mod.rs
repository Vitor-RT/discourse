pub mod discord_authenticator;
pub mod facebook_authenticator;
pub mod github_authenticator;
pub mod google_oauth2_authenticator;
pub mod twitter_authenticator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discord_authenticator() {
        assert_eq!(
            discord_authenticator::DiscordAuthenticator::name(),
            "discord"
        );
        assert_eq!(
            discord_authenticator::DiscordAuthenticator::display_name(),
            "Discord"
        );
        assert_eq!(
            discord_authenticator::DiscordAuthenticator::provider_url(),
            "https://discord.com"
        );
        assert_eq!(
            discord_authenticator::DiscordAuthenticator::enable_setting(),
            "enable_discord_logins"
        );
        assert_eq!(
            discord_authenticator::DiscordAuthenticator::required_settings(),
            vec!["discord_client_id", "discord_secret"]
        );
    }

    #[test]
    fn test_facebook_authenticator() {
        assert_eq!(
            facebook_authenticator::FacebookAuthenticator::name(),
            "facebook"
        );
        assert_eq!(
            facebook_authenticator::FacebookAuthenticator::display_name(),
            "Facebook"
        );
        assert_eq!(
            facebook_authenticator::FacebookAuthenticator::provider_url(),
            "https://www.facebook.com"
        );
        assert_eq!(
            facebook_authenticator::FacebookAuthenticator::enable_setting(),
            "enable_facebook_logins"
        );
        assert_eq!(
            facebook_authenticator::FacebookAuthenticator::required_settings(),
            vec!["facebook_app_id", "facebook_app_secret"]
        );
        assert_eq!(
            facebook_authenticator::FacebookAuthenticator::AVATAR_SIZE,
            480
        );
    }

    #[test]
    fn test_github_authenticator() {
        assert_eq!(github_authenticator::GithubAuthenticator::name(), "github");
        assert_eq!(
            github_authenticator::GithubAuthenticator::display_name(),
            "GitHub"
        );
        assert_eq!(
            github_authenticator::GithubAuthenticator::provider_url(),
            "https://github.com"
        );
        assert_eq!(
            github_authenticator::GithubAuthenticator::enable_setting(),
            "enable_github_logins"
        );
        assert_eq!(
            github_authenticator::GithubAuthenticator::required_settings(),
            vec!["github_client_id", "github_client_secret"]
        );
    }

    #[test]
    fn test_google_oauth2_authenticator() {
        assert_eq!(
            google_oauth2_authenticator::GoogleOauth2Authenticator::name(),
            "google_oauth2"
        );
        assert_eq!(
            google_oauth2_authenticator::GoogleOauth2Authenticator::display_name(),
            "Google"
        );
        assert_eq!(
            google_oauth2_authenticator::GoogleOauth2Authenticator::provider_url(),
            "https://accounts.google.com"
        );
        assert_eq!(
            google_oauth2_authenticator::GoogleOauth2Authenticator::enable_setting(),
            "enable_google_oauth2_logins"
        );
        assert_eq!(
            google_oauth2_authenticator::GoogleOauth2Authenticator::required_settings(),
            vec!["google_oauth2_client_id", "google_oauth2_client_secret"]
        );
        assert_eq!(
            google_oauth2_authenticator::GoogleOauth2Authenticator::GROUPS_SCOPE,
            "https://www.googleapis.com/auth/admin.directory.group.readonly"
        );
        assert_eq!(
            google_oauth2_authenticator::GoogleOauth2Authenticator::GROUPS_DOMAIN,
            "admin.googleapis.com"
        );
        assert_eq!(
            google_oauth2_authenticator::GoogleOauth2Authenticator::GROUPS_PATH,
            "/admin/directory/v1/groups"
        );
        assert_eq!(
            google_oauth2_authenticator::GoogleOauth2Authenticator::OAUTH2_BASE_URL,
            "https://oauth2.googleapis.com"
        );
    }

    #[test]
    fn test_twitter_authenticator() {
        assert_eq!(
            twitter_authenticator::TwitterAuthenticator::name(),
            "twitter"
        );
        assert_eq!(
            twitter_authenticator::TwitterAuthenticator::display_name(),
            "X / Twitter"
        );
        assert_eq!(
            twitter_authenticator::TwitterAuthenticator::provider_url(),
            "https://x.com"
        );
        assert_eq!(
            twitter_authenticator::TwitterAuthenticator::enable_setting(),
            "enable_twitter_logins"
        );
        assert_eq!(
            twitter_authenticator::TwitterAuthenticator::required_settings(),
            vec!["twitter_consumer_key", "twitter_consumer_secret"]
        );
    }
}
