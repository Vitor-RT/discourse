use std::collections::HashMap;

pub struct User {
    pub id: i32,
}

pub struct Topic {
    pub id: i32,
    pub tags: Vec<String>,
}

pub struct Post {
    pub id: i32,
}

pub struct Guardian;

impl Guardian {
    pub fn new(_user: &User) -> Self {
        Guardian
    }
}

pub struct TopicChanges<'a> {
    pub topic: &'a Topic,
    pub user: &'a User,
    pub silent: bool,
    changed: HashMap<String, (String, String)>,
    errored: bool,
}

impl<'a> TopicChanges<'a> {
    pub fn new(topic: &'a Topic, user: &'a User) -> Self {
        TopicChanges {
            topic,
            user,
            silent: false,
            changed: HashMap::new(),
            errored: false,
        }
    }

    pub fn errored(&self) -> bool {
        self.errored
    }

    pub fn check_result(&mut self, res: bool) {
        if !res {
            self.errored = true;
        }
    }

    pub fn record_change(&mut self, field_name: &str, previous_val: &str, new_val: &str) {
        if previous_val == new_val {
            return;
        }
        self.changed.insert(field_name.to_string(), (previous_val.to_string(), new_val.to_string()));
    }

    pub fn diff(&self) -> &HashMap<String, (String, String)> {
        &self.changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_change() {
        let topic = Topic { id: 1, tags: vec![] };
        let user = User { id: 1 };
        let mut changes = TopicChanges::new(&topic, &user);

        // Same value should not record
        changes.record_change("title", "old title", "old title");
        assert!(changes.diff().is_empty());

        // Different value should record
        changes.record_change("title", "old title", "new title");
        assert_eq!(changes.diff().len(), 1);
        let (old, new) = changes.diff().get("title").unwrap();
        assert_eq!(old, "old title");
        assert_eq!(new, "new title");
    }

    #[test]
    fn test_check_result() {
        let topic = Topic { id: 1, tags: vec![] };
        let user = User { id: 1 };
        let mut changes = TopicChanges::new(&topic, &user);

        assert!(!changes.errored());
        changes.check_result(true);
        assert!(!changes.errored());

        changes.check_result(false);
        assert!(changes.errored());
    }
}
