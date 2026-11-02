pub mod core_bundle;
pub mod helpers;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref DANGEROUS_BIDI_CHARACTERS: Vec<&'static str> = vec![
        "\u{202A}", "\u{202B}", "\u{202C}", "\u{202D}", "\u{202E}", "\u{2066}", "\u{2067}",
        "\u{2068}", "\u{2069}",
    ];
    pub static ref DANGEROUS_BIDI_REGEXP: Regex = {
        let pattern = DANGEROUS_BIDI_CHARACTERS.join("|");
        Regex::new(&pattern).unwrap()
    };
    pub static ref VIMEO_PLAYER_PATH: Regex = Regex::new(r"\A/video/(?P<id>\d+)/?\z").unwrap();
    pub static ref VIMEO_UNLISTED_HASH: Regex = Regex::new(r"\A[a-zA-Z0-9]+\z").unwrap();
}

pub const BLOCKED_HOTLINKED_SRC_ATTR: &str = "data-blocked-hotlinked-src";
pub const BLOCKED_HOTLINKED_SRCSET_ATTR: &str = "data-blocked-hotlinked-srcset";
pub const VIMEO_PLAYER_HOST: &str = "player.vimeo.com";

pub fn strip_bidi(text: &str) -> String {
    DANGEROUS_BIDI_REGEXP.replace_all(text, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_bidi_strip() {
        let input = "Hello\u{202A}World";
        assert_eq!(strip_bidi(input), "HelloWorld");
    }

    #[test]
    fn test_vimeo_player_path() {
        assert!(VIMEO_PLAYER_PATH.is_match("/video/12345/"));
        assert!(!VIMEO_PLAYER_PATH.is_match("/video/abc/"));
    }
}
