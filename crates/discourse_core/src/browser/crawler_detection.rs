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
    pub fn to_matcher(custom_agents: &str, is_test: bool) -> Regex {
        let mut escaped: Vec<String> = DEFAULT_CRAWLERS
            .iter()
            .map(|agent| regex::escape(agent))
            .collect();

        if is_test {
            escaped.push(regex::escape("Rails Testing"));
        }

        if !custom_agents.trim().is_empty() {
            for agent in custom_agents.split('|') {
                let trimmed = agent.trim();
                if !trimmed.is_empty() {
                    escaped.push(regex::escape(trimmed));
                }
            }
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
        let agent = match user_agent {
            Some(a) if !a.trim().is_empty() => a,
            _ => return true,
        };

        if !crawler_check_bypass_agents.trim().is_empty() {
            let bypass_re = Regex::new(&format!("(?i)({})", crawler_check_bypass_agents)).unwrap();
            if bypass_re.is_match(agent) {
                return false;
            }
        }

        if !non_crawler_user_agents.trim().is_empty() {
            let non_crawler_re = Regex::new(&format!("(?i)({})", non_crawler_user_agents)).unwrap();
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
            let blocked_re = Regex::new(&format!("(?i)({})", blocked_crawler_user_agents)).unwrap();
            if blocked_re.is_match(agent) {
                return false;
            }
        }

        if !allowed_crawler_user_agents.trim().is_empty() {
            let allowed_re = Regex::new(&format!("(?i)({})", allowed_crawler_user_agents)).unwrap();
            if allowed_re.is_match(agent) {
                return true;
            }
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
