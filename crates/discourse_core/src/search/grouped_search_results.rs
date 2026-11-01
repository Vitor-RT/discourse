use scraper::Html;
use html_escape::decode_html_entities;
use regex::Regex;

pub const BLURB_LENGTH: usize = 200;
pub const OMISSION: &str = "...";

pub fn scrub_html(html_str: &str) -> String {
    let document = Html::parse_fragment(html_str);

    // Simplistic scrubbing matching Nokogiri behavior
    let mut text = String::new();
    for node in document.tree.nodes() {
        if let scraper::node::Node::Text(t) = node.value() {
            text.push_str(&t.text);
            text.push(' '); // mimic characters appending
        }
    }

    let decoded = decode_html_entities(&text).to_string();
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(&decoded, " ").trim().to_string()
}

pub fn truncate(text: &str, length: usize) -> String {
    if text.chars().count() <= length {
        return text.to_string();
    }

    let mut truncated: String = text.chars().take(length).collect();
    truncated.push_str(OMISSION);
    truncated
}

pub fn excerpt(text: &str, phrase: &str, radius: usize) -> String {
    let text_lower = text.to_lowercase();
    let phrase_lower = phrase.to_lowercase();

    if let Some(pos) = text_lower.find(&phrase_lower) {
        let start = if pos > radius {
            pos - radius
        } else {
            0
        };

        let end = pos + phrase.len() + radius;
        let mut excerpt = String::new();

        let char_indices = text.char_indices();
        let mut actual_start = 0;
        let mut actual_end = text.len();

        for (i, (byte_idx, _)) in char_indices.enumerate() {
            if i == start {
                actual_start = byte_idx;
            }
            if i == end {
                actual_end = byte_idx;
                break;
            }
        }

        if start > 0 {
            excerpt.push_str(OMISSION);
        }
        excerpt.push_str(&text[actual_start..actual_end]);
        if end < text.chars().count() {
            excerpt.push_str(OMISSION);
        }
        excerpt
    } else {
        truncate(text, radius * 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello world", 5), "Hello...");
        assert_eq!(truncate("Hello", 5), "Hello");
        assert_eq!(truncate("Hello", 10), "Hello");
    }

    #[test]
    fn test_excerpt() {
        let text = "Hello world this is a test";
        assert_eq!(excerpt(text, "this", 5), "...orld this is a...");
    }

    #[test]
    fn test_scrub_html() {
        assert_eq!(scrub_html("<p>Hello</p>"), "Hello");
        assert_eq!(scrub_html("Hello <a href='foo'>world</a>"), "Hello world");
    }
}
