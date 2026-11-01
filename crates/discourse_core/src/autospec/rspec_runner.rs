use std::collections::HashSet;
use std::fs;
use std::path::Path;
use regex::Regex;

use super::base_runner::BaseRunner;

pub type CallbackFn = Box<dyn Fn(&regex::Captures) -> String + Send + Sync>;

pub struct WatcherRule {
    pub pattern: Regex,
    pub callback: Option<CallbackFn>,
}

pub struct RspecRunner {
    pub watchers: Vec<WatcherRule>,
    pub reloaders: HashSet<String>,
}

impl Default for RspecRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl RspecRunner {
    pub fn new() -> Self {
        let mut runner = RspecRunner {
            watchers: Vec::new(),
            reloaders: HashSet::new(),
        };
        runner.default_watchers();
        runner
    }

    pub fn watch_exact(&mut self, file: &str, target: Option<&str>) {
        let pattern = format!("^{}$", regex::escape(file));
        let target_str = target.map(|s| s.to_string());
        let callback: Option<CallbackFn> = match target_str {
            Some(t) => Some(Box::new(move |_| t.clone())),
            None => None,
        };
        if let Ok(reg) = Regex::new(&pattern) {
            self.watchers.push(WatcherRule { pattern: reg, callback });
        }
    }

    pub fn watch_regex(&mut self, pattern: &str, callback: Option<CallbackFn>) {
        if let Ok(reg) = Regex::new(pattern) {
            self.watchers.push(WatcherRule { pattern: reg, callback });
        }
    }

    pub fn default_watchers(&mut self) {
        self.watch_regex(r"^lib/(.+)\.rb$", Some(Box::new(|m: &regex::Captures| format!("spec/components/{}_spec.rb", &m[1]))));
        self.watch_regex(r"^app/(.+)\.rb$", Some(Box::new(|m: &regex::Captures| format!("spec/{}_spec.rb", &m[1]))));
        self.watch_regex(r"^app/(.+)(\.erb|\.haml)$", Some(Box::new(|m: &regex::Captures| format!("spec/{}{}_spec.rb", &m[1], &m[2]))));
        self.watch_regex(r"^spec/.+_spec\.rb$", None);
        self.watch_regex(r"^spec/support/.+\.rb$", Some(Box::new(|_| "spec".to_string())));

        self.watch_exact("app/controllers/application_controller.rb", Some("spec/requests"));
        self.watch_regex(r"app/controllers/(.+).rb", Some(Box::new(|m: &regex::Captures| format!("spec/requests/{}_spec.rb", &m[1]))));
        self.watch_regex(r"^app/views/(.+)/.+\.(erb|haml)$", Some(Box::new(|m: &regex::Captures| format!("spec/requests/{}_spec.rb", &m[1]))));

        self.watch_regex(r"^spec/fabricators/.+_fabricator\.rb$", Some(Box::new(|_| "spec".to_string())));

        self.watch_regex(r"^frontend/pretty-text/.*\.js\.es6$", Some(Box::new(|_| "spec/components/pretty_text_spec.rb".to_string())));
        self.watch_regex(r"^plugins/.*/discourse-markdown/.*\.js\.es6$", Some(Box::new(|_| "spec/components/pretty_text_spec.rb".to_string())));

        self.watch_regex(r"^plugins/.*/spec/.*\.rb", None);
        self.watch_regex(r"^(plugins/.*/)plugin\.rb", Some(Box::new(|m: &regex::Captures| format!("{}spec", &m[1]))));
        self.watch_regex(r"^(plugins/.*)/(lib|app)", Some(Box::new(|m: &regex::Captures| format!("{}/spec/integration", &m[1]))));
        self.watch_regex(r"^(plugins/.*)/lib/(.*)\.rb", Some(Box::new(|m: &regex::Captures| format!("{}/spec/lib/{}_spec.rb", &m[1], &m[2]))));

        self.watch_exact("spec/rails_helper.rb", None);
        self.watch_regex(r"config/.+\.rb", None);
    }
}

impl BaseRunner for RspecRunner {
    fn failed_specs(&self) -> Vec<String> {
        let path = Path::new("./tmp/rspec_result");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                return content.lines().map(|s| s.to_string()).collect();
            }
        }
        vec![]
    }
}
