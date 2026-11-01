pub struct DiscordAuthenticator;

impl DiscordAuthenticator {
    pub fn name() -> &'static str {
        "discord"
    }

    pub fn display_name() -> &'static str {
        "Discord"
    }

    pub fn provider_url() -> &'static str {
        "https://discord.com"
    }

    pub fn enable_setting() -> &'static str {
        "enable_discord_logins"
    }

    pub fn required_settings() -> Vec<&'static str> {
        vec!["discord_client_id", "discord_secret"]
    }
}
