use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref FILTER_ALIASES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("category", "category");
        m.insert("categories", "category");
        m.insert("exclude_category", "category");
        m.insert("exclude_categories", "category");
        m.insert("tag", "tag");
        m.insert("tags", "tag");
        m.insert("exclude_tag", "tag");
        m.insert("exclude_tags", "tag");
        m.insert("username", "usernames");
        m.insert("usernames", "usernames");
        m.insert("group", "groups");
        m.insert("groups", "groups");
        m.insert("topic", "topics");
        m.insert("topics", "topics");
        m.insert("created_after", "after");
        m.insert("created_before", "before");
        m.insert("topic_created_after", "topic_after");
        m.insert("topic_created_before", "topic_before");
        m
    };
}

pub const ORDER_VALUES: &[&str] = &["latest", "oldest", "latest_topic", "oldest_topic", "likes"];
pub const POST_TYPE_VALUES: &[&str] = &["regular", "all", "first", "reply", "moderator_action", "small_action", "whisper"];
pub const STORED_POST_TYPE_VALUES: &[&str] = &["regular", "moderator_action", "small_action", "whisper"];
pub const POST_TYPE_SCOPE_VALUES: &[&str] = &["regular", "moderator_action", "small_action", "whisper", "all"];
pub const STATUS_VALUES: &[&str] = &["open", "closed", "archived", "listed", "unlisted", "deleted", "public", "noreplies", "single_user"];

pub struct PostsFilter;

pub struct ParsedFilter {
    pub token: String,
    pub raw_key: String,
    pub key: String,
    pub prefix: String,
    pub value: String,
    pub exclude: bool,
    pub exact: bool,
}

impl PostsFilter {
    pub fn strip_surrounding_quotes(value: &str) -> String {
        let val = value.trim();
        if val.len() >= 2 {
            if (val.starts_with('"') && val.ends_with('"')) ||
               (val.starts_with('\'') && val.ends_with('\'')) {
                return val[1..val.len()-1].to_string();
            }
        }
        val.to_string()
    }

    pub fn parse_filter(word: &str) -> Option<ParsedFilter> {
        let re = regex::Regex::new(r"^(?P<prefix>-=|=-|-|=)?(?P<key>[\w-]+):(?P<value>.+)$").ok()?;
        let caps = re.captures(word)?;

        let prefix = caps.name("prefix").map_or("", |m| m.as_str());
        let raw_key = caps.name("key")?.as_str().to_lowercase();
        let value = caps.name("value")?.as_str();

        let key = FILTER_ALIASES.get(raw_key.as_str()).map_or(raw_key.clone(), |&v| v.to_string());

        let exclude = prefix.contains('-') || raw_key.starts_with("exclude_");
        let exact = prefix.contains('=');
        let stripped_value = Self::strip_surrounding_quotes(value.trim());

        Some(ParsedFilter {
            token: word.to_string(),
            raw_key,
            key,
            prefix: prefix.to_string(),
            value: stripped_value,
            exclude,
            exact,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_surrounding_quotes() {
        assert_eq!(PostsFilter::strip_surrounding_quotes(r#""hello""#), "hello");
        assert_eq!(PostsFilter::strip_surrounding_quotes("'hello'"), "hello");
        assert_eq!(PostsFilter::strip_surrounding_quotes("hello"), "hello");
        assert_eq!(PostsFilter::strip_surrounding_quotes(r#""hello"#), r#""hello"#);
    }

    #[test]
    fn test_parse_filter() {
        let filter1 = PostsFilter::parse_filter("category:bug").unwrap();
        assert_eq!(filter1.key, "category");
        assert_eq!(filter1.value, "bug");
        assert!(!filter1.exclude);
        assert!(!filter1.exact);

        let filter2 = PostsFilter::parse_filter("-tags:rust").unwrap();
        assert_eq!(filter2.key, "tag"); // mapped via alias
        assert_eq!(filter2.value, "rust");
        assert!(filter2.exclude);
        assert!(!filter2.exact);

        let filter3 = PostsFilter::parse_filter("=category:\"feature\"").unwrap();
        assert_eq!(filter3.key, "category");
        assert_eq!(filter3.value, "feature"); // stripped quotes
        assert!(!filter3.exclude);
        assert!(filter3.exact);

        let filter4 = PostsFilter::parse_filter("-=groups:'admins'").unwrap();
        assert_eq!(filter4.key, "groups");
        assert_eq!(filter4.value, "admins");
        assert!(filter4.exclude);
        assert!(filter4.exact);
    }
}
