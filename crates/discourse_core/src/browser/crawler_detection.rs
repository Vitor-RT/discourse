use regex::Regex;

pub const DEFAULT_CRAWLERS: &[&str] = &[
    "archive.org_bot",
    "baiduspider",
    "bingbot",
    "binlar",
    "casper",
    "checkpriv",
    "choppy",
    "clshttp",
    "cmsworld",
    "diavol",
    "dotbot",
    "extract",
    "feedfinder",
    "furl",
    "ghiro",
    "gsa-crawler",
    "gomez",
    "googlebot",
    "heartbeat",
    "httpdet",
    "internal dummy",
    "internetseer",
    "ip-web-crawler.com",
    "larbin",
    "looksmart",
    "mediapartners-google",
    "mqqbrowser",
    "netseer",
    "nutch",
    "lynx",
    "prowler",
    "seekbot",
    "sitelock",
    "sitemap",
    "site24x7",
    "siteuptime",
    "slurp",
    "sogou",
    "speedy",
    "spider",
    "sqworm",
    "surveybot",
    "teoma",
    "turnitin",
    "twengmo",
    "yandex",
    "yeti",
];

pub struct CrawlerDetection;

impl CrawlerDetection {
<<<<<<< HEAD
    pub fn to_matcher(custom_agents: &str, is_test: bool) -> Regex {
        let mut escaped: Vec<String> = DEFAULT_CRAWLERS
            .iter()
            .map(|agent| regex::escape(agent))
=======
    pub const WAYBACK_MACHINE_URL: &'static str = "archive.org";

    pub fn to_matcher(string: &str, is_real_and_test: bool) -> Regex {
        if string.is_empty() {
            // Return a regex that never matches anything
            // In rust regex `$` matches end of string, so matching `a^` is impossible
            return Regex::new(r"a^").unwrap();
        }

        let mut escaped: Vec<String> = string
            .split('|')
            .filter(|s| !s.is_empty())
            .map(regex::escape)
>>>>>>> origin/rust-migration-6812869939882800025
            .collect();

        if is_test {
            escaped.push(regex::escape("Rails Testing"));
        }

<<<<<<< HEAD
        if !custom_agents.trim().is_empty() {
            for agent in custom_agents.split('|') {
                let trimmed = agent.trim();
                if !trimmed.is_empty() {
                    escaped.push(regex::escape(trimmed));
                }
            }
=======
        if escaped.is_empty() {
            return Regex::new(r"a^").unwrap();
>>>>>>> origin/rust-migration-6812869939882800025
        }

        let pattern = format!("(?i)({})", escaped.join("|"));
        Regex::new(&pattern).unwrap()
    }

    pub fn is_crawler(
        user_agent: Option<&str>,
        via_header: Option<&str>,
        non_crawler_user_agents: &str,
        crawler_user_agents: &str,
        crawler_check_bypass_agents: &str,
        is_test: bool,
    ) -> bool {
<<<<<<< HEAD
        let agent = match user_agent {
            Some(a) if !a.trim().is_empty() => a,
            _ => return true,
        };
=======
        let ua_contains_wayback = user_agent
            .map(|ua| ua.contains(Self::WAYBACK_MACHINE_URL))
            .unwrap_or(false);
        let via_contains_wayback = via_header
            .map(|via| via.contains(Self::WAYBACK_MACHINE_URL))
            .unwrap_or(false);
>>>>>>> origin/rust-migration-6812869939882800025

        if !crawler_check_bypass_agents.trim().is_empty() {
            let bypass_re = Regex::new(&format!("(?i)({})", crawler_check_bypass_agents)).unwrap();
            if bypass_re.is_match(agent) {
                return false;
            }
        }

        if !non_crawler_user_agents.trim().is_empty() {
            let non_crawler_re =
                Regex::new(&format!("(?i)({})", non_crawler_user_agents)).unwrap();
            if non_crawler_re.is_match(agent) {
                return false;
            }
        }

        if let Some(via) = via_header {
            if via.to_lowercase().contains("prerender") {
                return true;
            }
        }

        let matcher = Self::to_matcher(crawler_user_agents, is_test);
        matcher.is_match(agent)
    }

    pub fn allow_crawler(
        user_agent: Option<&str>,
        allowed_crawler_user_agents: &str,
        blocked_crawler_user_agents: &str,
    ) -> bool {
        let agent = match user_agent {
            Some(a) => a,
            None => return false,
        };

        if !blocked_crawler_user_agents.trim().is_empty() {
            let blocked_re =
                Regex::new(&format!("(?i)({})", blocked_crawler_user_agents)).unwrap();
            if blocked_re.is_match(agent) {
                return false;
            }
        }

<<<<<<< HEAD
        if !allowed_crawler_user_agents.trim().is_empty() {
            let allowed_re =
                Regex::new(&format!("(?i)({})", allowed_crawler_user_agents)).unwrap();
            if allowed_re.is_match(agent) {
                return true;
            }
=======
        if !allowed_crawler_user_agents.is_empty() {
            let allowlisted = Self::to_matcher(allowed_crawler_user_agents, false);
            user_agent
                .map(|ua| allowlisted.is_match(ua))
                .unwrap_or(false)
        } else {
            let blocklisted = Self::to_matcher(blocked_crawler_user_agents, false);
            user_agent
                .map(|ua| !blocklisted.is_match(ua))
                .unwrap_or(true)
>>>>>>> origin/rust-migration-6812869939882800025
        }

        false
    }

    pub fn show_to_crawler(
        user_agent: Option<&str>,
        via_header: Option<&str>,
        non_crawler_user_agents: &str,
        crawler_user_agents: &str,
        crawler_check_bypass_agents: &str,
        allowed_crawler_user_agents: &str,
        blocked_crawler_user_agents: &str,
        is_test: bool,
    ) -> bool {
        Self::is_crawler(
            user_agent,
            via_header,
            non_crawler_user_agents,
            crawler_user_agents,
            crawler_check_bypass_agents,
            is_test,
        ) && !Self::allow_crawler(
            user_agent,
            allowed_crawler_user_agents,
            blocked_crawler_user_agents,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_matcher() {
        let matcher = CrawlerDetection::to_matcher("Googlebot|Bingbot", false);
        assert!(matcher
            .is_match("Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"));
        assert!(matcher
            .is_match("Mozilla/5.0 (compatible; Bingbot/2.0; +http://www.bing.com/bingbot.htm)"));
        assert!(!matcher.is_match("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"));

        let empty_matcher = CrawlerDetection::to_matcher("", false);
        assert!(!empty_matcher.is_match("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"));
    }

    #[test]
    fn test_is_crawler() {
        assert!(CrawlerDetection::is_crawler(
            Some("archive.org_bot"),
            None,
            "",
            "",
            "",
            false
        ));
        assert!(CrawlerDetection::is_crawler(None, None, "", "", "", false));
    }
}
