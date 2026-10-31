pub struct CrawlerScorer;

impl CrawlerScorer {
    pub const BOT_SCORE_THRESHOLD: u32 = 55;

    pub const AUTOMATION_UA_SCORE: u32 = 100;

    pub const KNOWN_ASN_SCORE: u32 = 30;
    pub const DATACENTER_ASN_SCORE: u32 = 10;
    pub const SINGLE_REQUEST_NO_REFERRER_SCORE: u32 = 10;
    pub const SINGLE_REQUEST_LOCALE_PARAM_BONUS: u32 = 5;
    pub const STALE_BROWSER_SCORE: u32 = 5;

    pub const VELOCITY_LOW: u32 = 150;
    pub const VELOCITY_MEDIUM: u32 = 300;
    pub const VELOCITY_HIGH: u32 = 600;
    pub const VELOCITY_LOW_SCORE: u32 = 15;
    pub const VELOCITY_MEDIUM_SCORE: u32 = 20;
    pub const VELOCITY_HIGH_SCORE: u32 = 50;

    pub const CHURN_LOW_MIN_SESSIONS: u32 = 5;
    pub const CHURN_HIGH_MIN_SESSIONS: u32 = 10;
    pub const CHURN_MAX_AVG_EVENTS: u32 = 2;
    pub const CHURN_LOW_SCORE: u32 = 10;
    pub const CHURN_HIGH_SCORE: u32 = 20;

    pub const RAPID_NAV_MIN_GAPS: u32 = 10;
    pub const RAPID_NAV_MAX_MEDIAN_SECONDS: u32 = 5;
    pub const RAPID_NAV_SCORE: u32 = 15;

    pub const IP_ROTATION_MIN_IPS: u32 = 3;
    pub const IP_ROTATION_MAX_SECONDS_PER_CHANGE: u32 = 300;
    pub const IP_ROTATION_SCORE: u32 = 30;

    pub const REFERRER_MIN_EVENTS: u32 = 5;
    pub const REFERRER_LOW_RATIO: f64 = 0.5;
    pub const REFERRER_HIGH_RATIO: f64 = 0.8;
    pub const REFERRER_LOW_SCORE: u32 = 5;
    pub const REFERRER_HIGH_SCORE: u32 = 10;

    pub const MISSING_ENGAGEMENT_LOW_RATIO: f64 = 0.5;
    pub const MISSING_ENGAGEMENT_HIGH_RATIO: f64 = 0.9;
    pub const MISSING_ENGAGEMENT_LOW_SCORE: u32 = 20;
    pub const MISSING_ENGAGEMENT_HIGH_SCORE: u32 = 40;
    // Note: ENGAGEMENT_LOOKBACK handling depends on context, skipping logic for time duration
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(CrawlerScorer::BOT_SCORE_THRESHOLD, 55);
        assert_eq!(CrawlerScorer::AUTOMATION_UA_SCORE, 100);
        assert_eq!(CrawlerScorer::KNOWN_ASN_SCORE, 30);
    }
}
