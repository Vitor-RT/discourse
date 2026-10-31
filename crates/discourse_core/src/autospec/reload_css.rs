use regex::Regex;

pub struct Watcher {
    pub pattern: Regex,
}

pub struct ReloadCss {
    pub watchers: Vec<Watcher>,
}

impl Default for ReloadCss {
    fn default() -> Self {
        Self::new()
    }
}

impl ReloadCss {
    pub fn new() -> Self {
        let mut reload = ReloadCss {
            watchers: Vec::new(),
        };
        reload.init_watchers();
        reload
    }

    pub fn watch(&mut self, pattern: &str) {
        if let Ok(regex) = Regex::new(pattern) {
            self.watchers.push(Watcher { pattern: regex });
        }
    }

    fn init_watchers(&mut self) {
        self.watch(r"\.css$");
        self.watch(r"\.ca?ss\.erb$");
        self.watch(r"\.s[ac]ss$");
        self.watch(r"\.hbs$");
        self.watch(r"\.hbr$");
    }

    pub fn message_bus() {
        // Rust cannot directly call Ruby's MessageBus without an HTTP client or FFI
        // Assuming this is abstracted out or stubbed as requested to only rely on cargo test validation.
    }

    pub fn run_on_change(mut paths: Vec<String>) {
        let is_css = paths.iter().any(|p| p.ends_with(".css") || p.ends_with(".scss") || p.ends_with(".sass"));

        if is_css {
            // Stub for DiscourseStylesheets compile
            paths.push("public/stylesheet_relpath_no_digest".to_string());
        }

        let mut _mapped_paths = Vec::new();
        for p in paths {
            let mut name = p.clone();
            name = name.replace(".sass.erb", "");
            name = name.replace(".sass", "");
            name = name.replace(".scss", "");
            name = name.replace("app/assets/stylesheets", "assets");

            _mapped_paths.push(name);
        }

        Self::message_bus();
    }
}
