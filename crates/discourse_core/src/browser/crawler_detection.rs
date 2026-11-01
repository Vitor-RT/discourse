use regex::Regex;

pub struct CrawlerDetection;

impl CrawlerDetection {
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
            .map(|agent| regex::escape(agent))
            .collect();

        if is_real_and_test {
            escaped.push(regex::escape("Rails Testing"));
        }

        if escaped.is_empty() {
            return Regex::new(r"a^").unwrap();
        }

        let pattern = escaped.join("|");
        Regex::new(&format!("(?i){}", pattern)).unwrap()
    }

    pub fn is_crawler(
        user_agent: Option<&str>,
        via_header: Option<&str>,
        non_crawler_user_agents: &str,
        crawler_user_agents: &str,
        crawler_check_bypass_agents: &str,
        is_test: bool,
    ) -> bool {
        let ua_contains_wayback = user_agent
            .map(|ua| ua.contains(Self::WAYBACK_MACHINE_URL))
            .unwrap_or(false);
        let via_contains_wayback = via_header
            .map(|via| via.contains(Self::WAYBACK_MACHINE_URL))
            .unwrap_or(false);

        if user_agent.is_none() || ua_contains_wayback || via_contains_wayback {
            return true;
        }

        let ua = user_agent.unwrap();
        let possibly_real = Self::to_matcher(non_crawler_user_agents, is_test);

        if possibly_real.is_match(ua) {
            let known_bots = Self::to_matcher(crawler_user_agents, false);
            if known_bots.is_match(ua) {
                let bypass = Self::to_matcher(crawler_check_bypass_agents, false);
                !bypass.is_match(ua)
            } else {
                false
            }
        } else {
            true
        }
    }

    pub fn show_browser_update(user_agent: &str, browser_update_user_agents: &str) -> bool {
        if browser_update_user_agents.is_empty() {
            return false;
        }
        let matcher = Self::to_matcher(browser_update_user_agents, false);
        matcher.is_match(user_agent)
    }

    pub fn allow_crawler(
        user_agent: Option<&str>,
        allowed_crawler_user_agents: &str,
        blocked_crawler_user_agents: &str,
    ) -> bool {
        if allowed_crawler_user_agents.is_empty() && blocked_crawler_user_agents.is_empty() {
            return true;
        }

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
        }
    }

    pub fn is_blocked_crawler(
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
