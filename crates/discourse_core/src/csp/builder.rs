use crate::csp::Directives;
use url::Url;

pub const EXTENDABLE_DIRECTIVES: &[&str] = &[
    "base-uri",
    "frame-ancestors",
    "manifest-src",
    "object-src",
    "script-src",
    "worker-src",
];

pub struct Builder {
    pub directives: Directives,
    pub base_url: String,
}

impl Builder {
    pub fn new(directives: Directives, base_url: String) -> Self {
        Self {
            directives,
            base_url,
        }
    }

    pub fn extend(&mut self, extension: &Directives) {
        for (directive, sources) in extension {
            self.extend_directive(Self::normalize_directive(directive), sources.clone());
        }
    }

    pub fn build(&self) -> String {
        let mut policy_parts = Vec::new();
        for (directive, sources) in &self.directives {
            if sources.is_empty() {
                policy_parts.push(directive.clone());
            } else {
                policy_parts.push(format!("{} {}", directive, sources.join(" ")));
            }
        }
        policy_parts.join("; ")
    }

    fn normalize_directive(directive: &str) -> String {
        directive.replace('_', "-")
    }

    fn normalize_source(base_url: &str, source: &str) -> String {
        if source.starts_with('/') {
            format!("{}{}", base_url, source)
        } else if Url::parse(source).is_err() {
            source.to_string()
        } else {
            source.to_string()
        }
    }

    fn extend_directive(&mut self, directive: String, mut sources: Vec<String>) {
        if !EXTENDABLE_DIRECTIVES.contains(&directive.as_str()) {
            return;
        }

        let base_url = self.base_url.clone();
        sources = sources
            .into_iter()
            .map(|s| Self::normalize_source(&base_url, &s))
            .collect();

        if directive == "script-src" || directive == "worker-src" {
            // Ruby logic: reject { |s| s == "'unsafe-inline'" || s == "'self'" || !s.start_with?("'") }
            // So keep if: s != "'unsafe-inline'" && s != "'self'" && s.starts_with("'")
            // Wait, but a URL source like "https://example.com" does NOT start with "'".
            // Let's re-read the ruby strictly:
            // sources = sources.reject { |s| s == "'unsafe-inline'" || s == "'self'" || !s.start_with?("'") }
            // Wait, this actually means it keeps ONLY things that DO start with "'" (but not unsafe-inline or self).
            // Actually, wait, it strips them because "strict-dynamic" ignores them in CSP3 anyway.
            // But if we want to allow URLs for backwards compat?
            // Actually, in CSP3 strict-dynamic, ALL allow-list sources (URLs) are ignored.
            // Oh, so it really does strip all URLs and only keeps other single-quote keywords like "'wasm-unsafe-eval'" or "'report-sample'" ?
            // Let's implement EXACTLY the ruby reject logic:
            // reject if `s == "'unsafe-inline'" || s == "'self'" || !s.starts_with("'")`
            // that is the same as retaining if `s != "'unsafe-inline'" && s != "'self'" && s.starts_with("'")`
            sources.retain(|s| s != "'unsafe-inline'" && s != "'self'" && s.starts_with('\''));
        }

        let entry = self
            .directives
            .entry(directive.clone())
            .or_insert_with(Vec::new);

        for source in sources {
            if source.contains(';') || source.chars().any(|c| c.is_whitespace()) {
                continue;
            }
            if !entry.contains(&source) {
                entry.push(source);
            }
        }

        if entry.len() > 1 {
            entry.retain(|s| s != "'none'");
        }
    }
}
