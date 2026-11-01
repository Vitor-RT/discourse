use scraper::{Html, Selector};

pub struct PreviewOptions {
    pub max_width: Option<u32>,
    pub allowed_iframe_regexes: Vec<String>,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            max_width: None,
            allowed_iframe_regexes: vec![".*".to_string()],
        }
    }
}

pub struct Preview {
    pub url: String,
    pub options: PreviewOptions,
}

impl Preview {
    pub fn new(url: &str, options: PreviewOptions) -> Self {
        Self {
            url: url.to_string(),
            options,
        }
    }

    pub fn process_html(&self, html: &str) -> String {
        if let Some(_max_width) = self.options.max_width {
            let _document = Html::parse_fragment(html);
            let _selector = Selector::parse("[width]").unwrap();
        }
        html.to_string()
    }

    pub fn sanitize(&self, html: &str) -> String {
        let builder = super::sanitize_config::SanitizeConfig::onebox();
        builder.clean(html).to_string()
    }

    pub fn to_s(&self) -> String {
        "".to_string()
    }
}
