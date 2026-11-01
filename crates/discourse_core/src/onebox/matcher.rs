use url::Url;

pub struct MatcherOptions {
    pub content_type: Option<String>,
    pub allowed_iframe_regexes: Vec<String>,
}

impl Default for MatcherOptions {
    fn default() -> Self {
        Self {
            content_type: None,
            allowed_iframe_regexes: vec![],
        }
    }
}

pub struct Matcher {
    uri: Option<Url>,
    _options: MatcherOptions,
}

impl Matcher {
    pub fn new(url: &str, options: MatcherOptions) -> Self {
        let uri = Url::parse(url).ok();
        Self {
            uri,
            _options: options,
        }
    }

    pub fn oneboxed(&self) -> Option<String> {
        let uri = self.uri.as_ref()?;
        if uri.host_str().is_none() {
            return None;
        }

        // This would interact with Onebox options and Engine
        None
    }
}
