pub mod sanitize_config;
pub mod status_check;
pub mod preview;
pub mod matcher;
pub mod engine;
pub mod helpers;
pub mod domain_checker;
pub mod view;
pub mod layout;

pub struct OneboxOptions {
    pub connect_timeout: u64,
    pub timeout: u64,
    pub max_download_kb: u64,
    pub allowed_ports: Vec<u16>,
    pub allowed_schemes: Vec<String>,
    pub redirect_limit: u32,
}

impl Default for OneboxOptions {
    fn default() -> Self {
        Self {
            connect_timeout: 5,
            timeout: 10,
            max_download_kb: 2048,
            allowed_ports: vec![80, 443],
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
            redirect_limit: 5,
        }
    }
}

pub struct Onebox;

impl Onebox {
    pub fn options() -> OneboxOptions {
        OneboxOptions::default()
    }

    pub fn check(url: &str) -> status_check::StatusCheck {
        status_check::StatusCheck::new(url)
    }

    pub fn preview(url: &str) -> preview::Preview {
        preview::Preview::new(url, preview::PreviewOptions::default())
    }

    pub fn has_matcher(url: &str) -> bool {
        let matcher = matcher::Matcher::new(url, matcher::MatcherOptions::default());
        matcher.oneboxed().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_defaults() {
        let options = Onebox::options();
        assert_eq!(options.connect_timeout, 5);
        assert_eq!(options.timeout, 10);
        assert_eq!(options.max_download_kb, 2048);
        assert_eq!(options.allowed_ports, vec![80, 443]);
        assert_eq!(options.allowed_schemes, vec!["http".to_string(), "https".to_string()]);
        assert_eq!(options.redirect_limit, 5);
    }
}
