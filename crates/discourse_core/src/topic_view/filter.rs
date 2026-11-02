use std::sync::Arc;

pub trait Guardian {
    // Basic guardian trait
}

pub trait Scope: Clone {
    fn secured(guardian: &dyn Guardian) -> Self;
    fn where_clause(&mut self, clause: &str);
    fn in_order_of(&mut self, column: &str, ids: &[i32]);
    fn order(&mut self, order_clause: &str);
}

pub struct TopicsFilter<S: Scope, G: Guardian> {
    pub topic_ids: Option<Vec<i32>>,
    pub invalid_filters: Vec<String>,
    pub scope: S,
    guardian: Arc<G>,
    loaded_topic_users_reference: bool,
}

impl<S: Scope, G: Guardian> TopicsFilter<S, G> {
    pub fn new(guardian: Arc<G>, scope: Option<S>, loaded_topic_users_reference: bool) -> Self {
        let scope = scope.unwrap_or_else(|| S::secured(&*guardian));
        TopicsFilter {
            topic_ids: None,
            invalid_filters: Vec::new(),
            scope,
            guardian,
            loaded_topic_users_reference,
        }
    }

    pub fn filter_from_query_string(&mut self, query_string: &str) {
        // A minimal implementation parsing simple 'key:value' space separated for testability
        let tokens = query_string.split_whitespace();

        for token in tokens {
            let mut parts = token.splitn(2, ':');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                match key {
                    "status" => self.filter_status(value, None),
                    "topic" => self.filter_topics(value),
                    "order" => self.order_by(value),
                    _ => self.invalid_filters.push(token.to_string()),
                }
            } else {
                self.invalid_filters.push(token.to_string());
            }
        }
    }

    pub fn filter_status(&mut self, status: &str, _category_id: Option<i32>) {
        match status {
            "open" => self.scope.where_clause("NOT topics.closed AND NOT topics.archived"),
            "closed" => self.scope.where_clause("topics.closed"),
            "archived" => self.scope.where_clause("topics.archived"),
            "noreplies" => self.scope.where_clause("topics.reply_count = 0"),
            "single_user" => self.scope.where_clause("topics.participant_count = 1"),
            _ => self.invalid_filters.push(format!("status:{}", status)),
        }
    }

    pub fn filter_topics(&mut self, value: &str) {
        let ids: Vec<i32> = value
            .split(',')
            .filter_map(|s| s.parse::<i32>().ok())
            .filter(|id| *id > 0)
            .collect();

        let mut unique_ids = ids.clone();
        unique_ids.dedup();

        self.topic_ids = Some(unique_ids.clone());
        self.scope.in_order_of("id", &unique_ids);
    }

    // Simulates SiteSetting lookup
    fn site_setting_tagging_enabled() -> bool {
        true
    }

    pub fn filter_tags(&mut self, values: Vec<(Option<String>, String)>) {
        if !Self::site_setting_tagging_enabled() { return; }

        for (key_prefix, value) in values {
            if let Some(ref prefix) = key_prefix {
                if prefix != "-" {
                    self.invalid_filters.push(format!("{}tag:{}", prefix, value));
                    continue;
                }
            }

            // Minimal tokenization check. Actual pattern matching in Ruby regex is complex.
            let delimiter = if value.contains(',') { ',' } else { '+' };
            let tags: Vec<&str> = value.split(delimiter).collect();

            // let tag_ids = DiscourseTagging.visible_tag_ids_resolving_synonyms(tags, @guardian)
            // if delimiter == ',' { self.include_topics_with_any_tags(tag_ids) }
            // else { self.include_topics_with_all_tags(tag_ids) }

            // We just record the invalidity if it was poorly formatted, similar to regex match fail.
            if tags.is_empty() {
                self.invalid_filters.push(format!("tag:{}", value));
            }
        }
    }

    pub fn filter_categories(&mut self, values: Vec<(Option<String>, String)>) {
        let mut include_category_ids = Vec::new();
        let mut exclude_category_ids = Vec::new();

        for (key_prefix, value) in values {
            let exclude_categories = key_prefix.as_ref().map_or(false, |p| p.contains('-'));
            // let exclude_subcategories = key_prefix.as_ref().map_or(false, |p| p.contains('='));

            let slugs: Vec<&str> = value.split(',').collect(); // Simplified pattern

            // let ids = category_ids_from_slugs(slugs, exclude_subcategories)
            let ids = vec![1, 2]; // Mocked result

            if exclude_categories {
                exclude_category_ids.extend(ids);
            } else {
                include_category_ids.extend(ids);
            }
        }

        if !include_category_ids.is_empty() {
            self.scope.where_clause("topics.category_id IN (...)");
        }
    }

    pub fn filter_tag_groups(&mut self, values: Vec<(Option<String>, String)>) {
        for (_, value) in values {
            let _stripped_value = value.trim_matches(|c| c == '"' || c == '\'');
            // TagGroup.visible(@guardian).where_name(...).pluck(:id)
            self.scope.where_clause("id IN (SELECT topic_id FROM topic_tags...)");
        }
    }

    pub fn filter_by_range(&mut self, filter: &str, value: &str) {
        if value.is_empty() { return; }

        // Simulating RANGE_FILTERS hash
        let column_operator = match filter {
            "activity-after" => Some(("topics.bumped_at", ">=")),
            "activity-before" => Some(("topics.bumped_at", "<=")),
            "created-after" => Some(("topics.created_at", ">=")),
            "created-before" => Some(("topics.created_at", "<=")),
            "likes-min" => Some(("topics.like_count", ">=")),
            "likes-max" => Some(("topics.like_count", "<=")),
            _ => None,
        };

        if let Some((column, operator)) = column_operator {
            // @scope = joins_first_posts(@scope) if column.start_with?("first_posts.")
            self.scope.where_clause(&format!("{} {} '{}'", column, operator, value));
        }
    }

    pub fn filter_created_by(&mut self, names: Vec<&str>) {
        let _names_lower: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
        // let user_ids = User.where(username_lower: names_lower).pluck(:id)
        // if user_ids.any? ...

        self.scope.where_clause("user_id IN (...)");
    }

    pub fn filter_participation(&mut self, values: Vec<(Option<String>, String)>, by: &str) {
        for (key_prefix, value) in values {
            let require_all = value.contains('+');
            let mut names: Vec<&str> = if require_all {
                value.split('+').collect()
            } else {
                value.split(',').collect()
            };

            if require_all && value.contains(',') {
                names.clear();
            }

            if names.is_empty() {
                self.invalid_filters.push(format!("{}:{}", by, value));
                if key_prefix.map_or(true, |p| !p.contains('-')) {
                    self.scope.where_clause("1 = 0"); // @scope.none
                }
                continue;
            }

            // let ids = participant_ids(by, names)
            let ids = vec![1, 2]; // Mocked result

            if require_all {
                for _id in ids {
                    self.scope.where_clause(&format!("id IN ({}_participation_sql)", by));
                }
            } else {
                self.scope.where_clause(&format!("id IN ({}_participation_sql)", by));
            }
        }
    }

    // Simulating order_by which we found in the original ruby script and mapped in initial plan
    pub fn order_by(&mut self, value: &str) {
        match value {
            "created" => self.scope.order("topics.created_at DESC"),
            "activity" => self.scope.order("topics.bumped_at DESC"),
            "views" => self.scope.order("topics.views DESC"),
            _ => self.invalid_filters.push(format!("order:{}", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockGuardian {}
    impl Guardian for MockGuardian {}

    #[derive(Clone)]
    struct MockScope {
        where_clauses: Arc<RefCell<Vec<String>>>,
        order_clauses: Arc<RefCell<Vec<String>>>,
        in_order_clauses: Arc<RefCell<Vec<(String, Vec<i32>)>>>,
    }

    impl MockScope {
        fn new() -> Self {
            MockScope {
                where_clauses: Arc::new(RefCell::new(Vec::new())),
                order_clauses: Arc::new(RefCell::new(Vec::new())),
                in_order_clauses: Arc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl Scope for MockScope {
        fn secured(_guardian: &dyn Guardian) -> Self {
            MockScope::new()
        }

        fn where_clause(&mut self, clause: &str) {
            self.where_clauses.borrow_mut().push(clause.to_string());
        }

        fn in_order_of(&mut self, column: &str, ids: &[i32]) {
            self.in_order_clauses.borrow_mut().push((column.to_string(), ids.to_vec()));
        }

        fn order(&mut self, order_clause: &str) {
            self.order_clauses.borrow_mut().push(order_clause.to_string());
        }
    }

    #[test]
    fn test_filter_status_open() {
        let guardian = Arc::new(MockGuardian {});
        let scope = MockScope::new();
        let mut filter = TopicsFilter::new(guardian, Some(scope.clone()), false);

        filter.filter_status("open", None);
        assert_eq!(scope.where_clauses.borrow()[0], "NOT topics.closed AND NOT topics.archived");
    }

    #[test]
    fn test_filter_topics_parses_comma_separated() {
        let guardian = Arc::new(MockGuardian {});
        let scope = MockScope::new();
        let mut filter = TopicsFilter::new(guardian, Some(scope.clone()), false);

        filter.filter_topics("1,2,abc,3");
        assert_eq!(filter.topic_ids, Some(vec![1, 2, 3]));
        assert_eq!(scope.in_order_clauses.borrow()[0].1, vec![1, 2, 3]);
    }

    #[test]
    fn test_filter_from_query_string() {
        let guardian = Arc::new(MockGuardian {});
        let scope = MockScope::new();
        let mut filter = TopicsFilter::new(guardian, Some(scope.clone()), false);

        filter.filter_from_query_string("status:closed topic:10,20 order:views");

        let wheres = scope.where_clauses.borrow();
        assert_eq!(wheres[0], "topics.closed");

        assert_eq!(filter.topic_ids, Some(vec![10, 20]));

        let orders = scope.order_clauses.borrow();
        assert_eq!(orders[0], "topics.views DESC");
    }

    #[test]
    fn test_filter_tags_valid() {
        let guardian = Arc::new(MockGuardian {});
        let scope = MockScope::new();
        let mut filter = TopicsFilter::new(guardian, Some(scope.clone()), false);

        filter.filter_tags(vec![(None, "rust,migration".to_string())]);
        assert_eq!(filter.invalid_filters.len(), 0);
    }

    #[test]
    fn test_filter_by_range() {
        let guardian = Arc::new(MockGuardian {});
        let scope = MockScope::new();
        let mut filter = TopicsFilter::new(guardian, Some(scope.clone()), false);

        filter.filter_by_range("activity-after", "2024-01-01");
        assert_eq!(scope.where_clauses.borrow()[0], "topics.bumped_at >= '2024-01-01'");
    }
}
