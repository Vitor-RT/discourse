pub struct Helpers;

impl Helpers {
    pub fn clean(html: &str) -> String {
        let re = regex::Regex::new(r"<[^>]+>").unwrap();
        let cleaned = re.replace_all(html, " ");
        cleaned.replace("\n", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean() {
        assert_eq!(Helpers::clean("<p>Hello</p>"), " Hello ");
        assert_eq!(Helpers::clean("A\nB"), "AB");
    }
}
