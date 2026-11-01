use regex::Regex;
use std::sync::OnceLock;
use url::form_urlencoded;

pub const MAX_LENGTH: usize = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlugMethod {
    Ascii,
    Encoded,
    None,
}

impl Default for SlugMethod {
    fn default() -> Self {
        Self::Ascii
    }
}

pub struct Slug;

impl Slug {
    pub fn for_string(
        string: &str,
        default: &str,
        mut max_length: usize,
        method: Option<SlugMethod>,
    ) -> String {
        let method = method.unwrap_or_default();
        if method == SlugMethod::Encoded {
            max_length = 9999;
        }

        static EMOJI_CODE_RE: OnceLock<Regex> = OnceLock::new();
        let emoji_re = EMOJI_CODE_RE.get_or_init(|| Regex::new(r":[\w\-+]+:").unwrap());
        let mut text = string.to_string();
        if !text.is_empty() {
            text = emoji_re.replace_all(&text, "").into_owned();
        }

        let mut slug = match method {
            SlugMethod::Ascii => Self::ascii_generator(&text),
            SlugMethod::Encoded => Self::encoded_generator(&text, true),
            SlugMethod::None => Self::none_generator(&text),
        };

        slug = Self::prettify_slug(&slug, max_length);

        if slug.trim().is_empty() || Self::slug_is_only_numbers(&slug) {
            default.to_string()
        } else {
            slug
        }
    }

    fn slug_is_only_numbers(slug: &str) -> bool {
        slug.chars().all(|c| c.is_ascii_digit())
    }

    fn prettify_slug(slug: &str, max_length: usize) -> String {
        if Self::slug_is_only_numbers(slug) {
            return String::new();
        }

        let mut pretty = slug.replace('_', "-");

        if pretty.len() > max_length {
            if pretty.is_char_boundary(max_length) {
                pretty.truncate(max_length);
            } else {
                let mut valid_len = max_length;
                while valid_len > 0 && !pretty.is_char_boundary(valid_len) {
                    valid_len -= 1;
                }
                pretty.truncate(valid_len);
            }
        }

        static SQUEEZE_RE: OnceLock<Regex> = OnceLock::new();
        let squeeze_re = SQUEEZE_RE.get_or_init(|| Regex::new(r"-+").unwrap());
        pretty = squeeze_re.replace_all(&pretty, "-").into_owned();

        static BOUNDARY_RE: OnceLock<Regex> = OnceLock::new();
        let boundary_re = BOUNDARY_RE.get_or_init(|| Regex::new(r"^-+|-+$").unwrap());
        boundary_re.replace_all(&pretty, "").into_owned()
    }

    fn ascii_generator(string: &str) -> String {
        let string_no_quotes = any_ascii::any_ascii(&string.replace('\'', ""));

        static NON_ALNUM_RE: OnceLock<Regex> = OnceLock::new();
        let re = NON_ALNUM_RE.get_or_init(|| Regex::new(r"[^a-zA-Z0-9\-]+").unwrap());

        re.replace_all(&string_no_quotes, "-").to_lowercase()
    }

    fn encoded_generator(string: &str, downcase: bool) -> String {
        static CHAR_FILTER_RE: OnceLock<Regex> = OnceLock::new();
        let filter_re = CHAR_FILTER_RE.get_or_init(|| {
            Regex::new(r#"[:/\?#\[\]@!\$&'\(\)\*\+,;=_\.~%\\`^\s|\{\}"<>]+"#).unwrap()
        });

        static SPACE_RE: OnceLock<Regex> = OnceLock::new();
        let space_re = SPACE_RE.get_or_init(|| Regex::new(r"\s+").unwrap());

        let mut processed = string.trim().to_string();
        processed = space_re.replace_all(&processed, "-").into_owned();
        processed = filter_re.replace_all(&processed, "").into_owned();

        if downcase {
            processed = processed.to_lowercase();
        }

        form_urlencoded::byte_serialize(processed.as_bytes()).collect()
    }

    fn none_generator(_string: &str) -> String {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug_for_ascii() {
        assert_eq!(Slug::for_string("Hello World!", "topic", MAX_LENGTH, Some(SlugMethod::Ascii)), "hello-world");
        assert_eq!(Slug::for_string("  Hello   World  ", "topic", MAX_LENGTH, Some(SlugMethod::Ascii)), "hello-world");
        assert_eq!(Slug::for_string("12345", "topic", MAX_LENGTH, Some(SlugMethod::Ascii)), "topic");
        assert_eq!(Slug::for_string("Bob's test", "topic", MAX_LENGTH, Some(SlugMethod::Ascii)), "bobs-test");
        assert_eq!(Slug::for_string("café", "topic", MAX_LENGTH, Some(SlugMethod::Ascii)), "cafe");
    }

    #[test]
    fn test_slug_for_encoded() {
        assert_eq!(Slug::for_string("Hello World!", "topic", MAX_LENGTH, Some(SlugMethod::Encoded)), "hello-world");
        assert_eq!(Slug::for_string("test:/:/test", "topic", MAX_LENGTH, Some(SlugMethod::Encoded)), "testtest");
    }

    #[test]
    fn test_slug_for_none() {
        assert_eq!(Slug::for_string("Hello World!", "topic", MAX_LENGTH, Some(SlugMethod::None)), "topic");
    }

    #[test]
    fn test_emoji_removal() {
        assert_eq!(Slug::for_string("Hello :smile: World", "topic", MAX_LENGTH, Some(SlugMethod::Ascii)), "hello-world");
    }
}
