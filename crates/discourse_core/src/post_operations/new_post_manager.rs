use std::collections::HashMap;

pub struct User {
    pub post_count: i32,
    pub topic_count: i32,
    pub trust_level: i32,
}

pub struct WordWatcher;

impl WordWatcher {
    pub fn new(_text: &str) -> Self {
        WordWatcher
    }

    pub fn should_silence(&self) -> bool {
        false // mock implementation
    }
}

pub struct NewPostManager {
    pub user: User,
    pub args: HashMap<String, String>,
}

impl NewPostManager {
    pub fn is_first_post(&self) -> bool {
        let first_post_checks = self.args.get("first_post_checks").map(|v| v == "true").unwrap_or(false);
        first_post_checks && self.user.post_count == 0 && self.user.topic_count == 0
    }

    pub fn is_fast_typer(&self, threshold: i32) -> bool {
        if !self.is_first_post() {
            return false;
        }

        let typing_duration = self.args.get("typing_duration_msecs")
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(0);

        typing_duration < threshold
    }

    pub fn auto_silence(&self) -> bool {
        if !self.is_first_post() {
            return false;
        }

        let title = self.args.get("title").map(|s| s.as_str()).unwrap_or("");
        let raw = self.args.get("raw").map(|s| s.as_str()).unwrap_or("");
        let content = format!("{} {}", title, raw);

        WordWatcher::new(&content).should_silence()
    }

    pub fn matches_auto_silence_regex(&self, regex_pattern: Option<&str>) -> bool {
        if !self.is_first_post() {
            return false;
        }

        let pattern = match regex_pattern {
            Some(p) if !p.is_empty() => p,
            _ => return false,
        };

        let title = self.args.get("title").map(|s| s.as_str()).unwrap_or("");
        let raw = self.args.get("raw").map(|s| s.as_str()).unwrap_or("");
        let content = format!("{} {}", title, raw);

        if let Ok(re) = regex::RegexBuilder::new(pattern).case_insensitive(true).build() {
            re.is_match(&content)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn setup_manager(args_map: HashMap<&str, &str>, user: User) -> NewPostManager {
        let mut args = HashMap::new();
        for (k, v) in args_map {
            args.insert(k.to_string(), v.to_string());
        }
        NewPostManager { user, args }
    }

    #[test]
    fn test_is_first_post() {
        let user = User { post_count: 0, topic_count: 0, trust_level: 0 };
        let mut args = HashMap::new();
        args.insert("first_post_checks", "true");
        let manager = setup_manager(args, user);
        assert!(manager.is_first_post());

        let mut args_false = HashMap::new();
        args_false.insert("first_post_checks", "false");
        let user2 = User { post_count: 0, topic_count: 0, trust_level: 0 };
        let manager2 = setup_manager(args_false, user2);
        assert!(!manager2.is_first_post());
    }

    #[test]
    fn test_is_fast_typer() {
        let user = User { post_count: 0, topic_count: 0, trust_level: 0 };
        let mut args = HashMap::new();
        args.insert("first_post_checks", "true");
        args.insert("typing_duration_msecs", "500");
        let manager = setup_manager(args, user);

        assert!(manager.is_fast_typer(1000));
        assert!(!manager.is_fast_typer(100));
    }
}

pub struct Post { pub id: i32 }

pub struct NewPostResult {
    pub success: bool,
    pub errors: Vec<String>,
}

impl NewPostResult {
    pub fn new(_status: &str, success: bool) -> Self {
        NewPostResult { success, errors: Vec::new() }
    }
}

pub struct PostCreator;
impl PostCreator {
    pub fn new(_user: &User, _args: &HashMap<String, String>) -> Self {
        PostCreator
    }

    pub fn create(&self) -> Post {
        Post { id: 1 }
    }

    pub fn spam(&self) -> bool {
        false
    }
}

impl NewPostManager {
    pub fn perform(&self) -> NewPostResult {
        // Mock perform logic
        let mut result = NewPostResult::new("create_post", true);
        let creator = PostCreator::new(&self.user, &self.args);
        let _post = creator.create();

        if creator.spam() {
            // flag spam
            result.success = false;
        }

        result
    }
}
