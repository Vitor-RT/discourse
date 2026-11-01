use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Default, Clone)]
pub struct TextCleanerOptions {
    pub deduplicate_exclamation_marks: bool,
    pub deduplicate_question_marks: bool,
    pub replace_all_upper_case: bool,
    pub capitalize_first_letter: bool,
    pub remove_all_periods_from_the_end: bool,
    pub remove_extraneous_space: bool,
    pub fixes_interior_spaces: bool,
    pub strip_whitespaces: bool,
    pub strip_zero_width_spaces: bool,
    pub is_turkic_locale: bool,
}

impl TextCleanerOptions {
    pub fn title_options(
        title_prettify: bool,
        allow_uppercase_posts: bool,
        title_remove_extraneous_space: bool,
        is_turkic_locale: bool,
    ) -> Self {
        Self {
            deduplicate_exclamation_marks: title_prettify,
            deduplicate_question_marks: title_prettify,
            replace_all_upper_case: title_prettify && !allow_uppercase_posts,
            capitalize_first_letter: title_prettify,
            remove_all_periods_from_the_end: title_prettify,
            remove_extraneous_space: title_prettify && title_remove_extraneous_space,
            fixes_interior_spaces: true,
            strip_whitespaces: true,
            strip_zero_width_spaces: true,
            is_turkic_locale,
        }
    }
}

pub struct TextCleaner;

impl TextCleaner {
    pub fn clean_title(title: &str, opts: &TextCleanerOptions) -> String {
        Self::clean(title, opts)
    }

    pub fn clean(text: &str, opts: &TextCleanerOptions) -> String {
        let mut text_owned = text.to_string();

        if opts.deduplicate_exclamation_marks {
            static EXCLAMATION_RE: OnceLock<Regex> = OnceLock::new();
            let re = EXCLAMATION_RE.get_or_init(|| Regex::new(r"!+").unwrap());
            text_owned = re.replace_all(&text_owned, "!").into_owned();
        }

        if opts.deduplicate_question_marks {
            static QUESTION_RE: OnceLock<Regex> = OnceLock::new();
            let re = QUESTION_RE.get_or_init(|| Regex::new(r"\?+").unwrap());
            text_owned = re.replace_all(&text_owned, "?").into_owned();
        }

        if opts.replace_all_upper_case && text_owned == Self::upcase(&text_owned, opts) {
            text_owned = Self::downcase(&text_owned, opts);
        }

        if opts.capitalize_first_letter {
            let mut parts = text_owned.splitn(2, ' ');
            let first = parts.next().unwrap_or("");
            let rest = parts.next();

            if !first.is_empty() && first == Self::downcase(first, opts) {
                let mut capitalized_first = Self::capitalize(first, opts);
                if let Some(r) = rest {
                    capitalized_first.push(' ');
                    capitalized_first.push_str(r);
                }
                text_owned = capitalized_first;
            }
        }

        if opts.remove_all_periods_from_the_end {
            static PERIOD_RE: OnceLock<Regex> = OnceLock::new();
            let re = PERIOD_RE.get_or_init(|| Regex::new(r"([^.])\.+(\s*)$").unwrap());
            text_owned = re.replace_all(&text_owned, "${1}${2}").into_owned();
        }

        if opts.remove_extraneous_space {
            static SPACE_RE: OnceLock<Regex> = OnceLock::new();
            let re = SPACE_RE.get_or_init(|| Regex::new(r"\s+([!?]\s*)$").unwrap());
            text_owned = re.replace_all(&text_owned, "${1}").into_owned();
        }

        if opts.fixes_interior_spaces {
            static INTERIOR_SPACE_RE: OnceLock<Regex> = OnceLock::new();
            let re = INTERIOR_SPACE_RE.get_or_init(|| Regex::new(r" +").unwrap());
            text_owned = re.replace_all(&text_owned, " ").into_owned();
        }

        text_owned = Self::normalize_whitespaces(&text_owned);

        if opts.strip_whitespaces {
            text_owned = text_owned.trim().to_string();
        }

        if opts.strip_zero_width_spaces {
            text_owned = text_owned.replace('\u{200B}', "");
        }

        text_owned
    }

    pub fn normalize_whitespaces(text: &str) -> String {
        static WHITESPACES_RE: OnceLock<Regex> = OnceLock::new();
        let re = WHITESPACES_RE.get_or_init(|| {
            Regex::new(r"(?i)(\u{00A0}|\u{1680}|\u{180E}|[\u{2000}-\u{200A}]|\u{2028}|\u{2029}|\u{202F}|\u{205F}|\u{3000})").unwrap()
        });
        re.replace_all(text, " ").into_owned()
    }

    pub fn downcase(text: &str, _opts: &TextCleanerOptions) -> String {
        text.to_lowercase()
    }

    pub fn upcase(text: &str, _opts: &TextCleanerOptions) -> String {
        text.to_uppercase()
    }

    pub fn capitalize(text: &str, _opts: &TextCleanerOptions) -> String {
        let mut chars = text.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().chain(chars).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_exclamation_marks() {
        let opts = TextCleanerOptions { deduplicate_exclamation_marks: true, ..Default::default() };
        assert_eq!(TextCleaner::clean("hello!!!!", &opts), "hello!");
    }

    #[test]
    fn test_deduplicate_question_marks() {
        let opts = TextCleanerOptions { deduplicate_question_marks: true, ..Default::default() };
        assert_eq!(TextCleaner::clean("what?????", &opts), "what?");
    }

    #[test]
    fn test_replace_all_upper_case() {
        let opts = TextCleanerOptions { replace_all_upper_case: true, ..Default::default() };
        assert_eq!(TextCleaner::clean("HELLO", &opts), "hello");
        assert_eq!(TextCleaner::clean("HELLo", &opts), "HELLo");
    }

    #[test]
    fn test_capitalize_first_letter() {
        let opts = TextCleanerOptions { capitalize_first_letter: true, ..Default::default() };
        assert_eq!(TextCleaner::clean("hello world", &opts), "Hello world");
        assert_eq!(TextCleaner::clean("Hello world", &opts), "Hello world");
        assert_eq!(TextCleaner::clean("hEllo world", &opts), "hEllo world");
    }

    #[test]
    fn test_remove_all_periods_from_the_end() {
        let opts = TextCleanerOptions { remove_all_periods_from_the_end: true, ..Default::default() };
        assert_eq!(TextCleaner::clean("hello...", &opts), "hello");
        assert_eq!(TextCleaner::clean("hello...  ", &opts), "hello  ");
        assert_eq!(TextCleaner::clean("hello", &opts), "hello");
    }

    #[test]
    fn test_remove_extraneous_space() {
        let opts = TextCleanerOptions { remove_extraneous_space: true, ..Default::default() };
        assert_eq!(TextCleaner::clean("hello !", &opts), "hello!");
        assert_eq!(TextCleaner::clean("what ?", &opts), "what?");
        assert_eq!(TextCleaner::clean("what ?  ", &opts), "what?  ");
    }

    #[test]
    fn test_fixes_interior_spaces() {
        let opts = TextCleanerOptions { fixes_interior_spaces: true, ..Default::default() };
        assert_eq!(TextCleaner::clean("hello   world", &opts), "hello world");
    }

    #[test]
    fn test_strip_whitespaces() {
        let opts = TextCleanerOptions { strip_whitespaces: true, ..Default::default() };
        assert_eq!(TextCleaner::clean("  hello  ", &opts), "hello");
    }

    #[test]
    fn test_strip_zero_width_spaces() {
        let opts = TextCleanerOptions { strip_zero_width_spaces: true, ..Default::default() };
        assert_eq!(TextCleaner::clean("hello\u{200B}world", &opts), "helloworld");
    }

    #[test]
    fn test_normalize_whitespaces() {
        assert_eq!(TextCleaner::normalize_whitespaces("hello\u{00A0}world"), "hello world");
    }
}
