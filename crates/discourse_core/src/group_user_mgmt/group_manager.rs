pub trait Database {
    fn add_users_to_group(&self, group_id: i32, user_ids: &[i32], notification_level: i32) -> Vec<i32>;
    fn remove_users_from_group(&self, group_id: i32, user_ids: &[i32]);
    fn trigger_user_added_event(&self, user_id: i32, automatic: bool);
    fn trigger_user_removed_event(&self, user_id: i32);
    fn sync_add_side_effects(&self, group_id: i32, added_user_ids: &[i32]);
    fn sync_removal_side_effects(&self, group_id: i32, removed_user_ids: &[i32]);
}

pub struct Group {
    pub id: i32,
    pub default_notification_level: i32,
}

pub struct GroupManager<'a, D: Database> {
    group: Group,
    db: &'a D,
}

impl<'a, D: Database> GroupManager<'a, D> {
    pub fn new(group: Group, db: &'a D) -> Self {
        Self { group, db }
    }

    pub fn add(&self, user_ids: &[i32], automatic: bool) -> Vec<i32> {
        if user_ids.is_empty() {
            return vec![];
        }

        let added_user_ids = self.db.add_users_to_group(self.group.id, user_ids, self.group.default_notification_level);

        if added_user_ids.is_empty() {
            return vec![];
        }

        self.db.sync_add_side_effects(self.group.id, &added_user_ids);

        for user_id in &added_user_ids {
            self.db.trigger_user_added_event(*user_id, automatic);
        }

        added_user_ids
    }

    pub fn remove(&self, user_ids: &[i32]) -> Vec<i32> {
        if user_ids.is_empty() {
            return vec![];
        }

        self.db.remove_users_from_group(self.group.id, user_ids);
        self.db.sync_removal_side_effects(self.group.id, user_ids);

        for user_id in user_ids {
            self.db.trigger_user_removed_event(*user_id);
        }

        user_ids.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockDatabase {
        added_users: RefCell<Vec<(i32, Vec<i32>, i32)>>,
        removed_users: RefCell<Vec<(i32, Vec<i32>)>>,
        triggered_added_events: RefCell<Vec<(i32, bool)>>,
        triggered_removed_events: RefCell<Vec<i32>>,
        synced_add_side_effects: RefCell<Vec<(i32, Vec<i32>)>>,
        synced_removal_side_effects: RefCell<Vec<(i32, Vec<i32>)>>,
    }

    impl MockDatabase {
        fn new() -> Self {
            Self {
                added_users: RefCell::new(vec![]),
                removed_users: RefCell::new(vec![]),
                triggered_added_events: RefCell::new(vec![]),
                triggered_removed_events: RefCell::new(vec![]),
                synced_add_side_effects: RefCell::new(vec![]),
                synced_removal_side_effects: RefCell::new(vec![]),
            }
        }
    }

    impl Database for MockDatabase {
        fn add_users_to_group(&self, group_id: i32, user_ids: &[i32], notification_level: i32) -> Vec<i32> {
            self.added_users.borrow_mut().push((group_id, user_ids.to_vec(), notification_level));
            user_ids.to_vec() // Mock assuming all were added successfully
        }

        fn remove_users_from_group(&self, group_id: i32, user_ids: &[i32]) {
            self.removed_users.borrow_mut().push((group_id, user_ids.to_vec()));
        }

        fn trigger_user_added_event(&self, user_id: i32, automatic: bool) {
            self.triggered_added_events.borrow_mut().push((user_id, automatic));
        }

        fn trigger_user_removed_event(&self, user_id: i32) {
            self.triggered_removed_events.borrow_mut().push(user_id);
        }

        fn sync_add_side_effects(&self, group_id: i32, added_user_ids: &[i32]) {
            self.synced_add_side_effects.borrow_mut().push((group_id, added_user_ids.to_vec()));
        }

        fn sync_removal_side_effects(&self, group_id: i32, removed_user_ids: &[i32]) {
            self.synced_removal_side_effects.borrow_mut().push((group_id, removed_user_ids.to_vec()));
        }
    }

    #[test]
    fn test_add_empty_user_ids() {
        let db = MockDatabase::new();
        let group = Group { id: 1, default_notification_level: 2 };
        let manager = GroupManager::new(group, &db);

        let added = manager.add(&[], false);
        assert!(added.is_empty());
        assert!(db.added_users.borrow().is_empty());
    }

    #[test]
    fn test_add_users() {
        let db = MockDatabase::new();
        let group = Group { id: 1, default_notification_level: 2 };
        let manager = GroupManager::new(group, &db);

        let user_ids = vec![10, 20];
        let added = manager.add(&user_ids, true);

        assert_eq!(added, user_ids);

        let added_records = db.added_users.borrow();
        assert_eq!(added_records.len(), 1);
        assert_eq!(added_records[0], (1, vec![10, 20], 2));

        let sync_add = db.synced_add_side_effects.borrow();
        assert_eq!(sync_add.len(), 1);
        assert_eq!(sync_add[0], (1, vec![10, 20]));

        let events = db.triggered_added_events.borrow();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], (10, true));
        assert_eq!(events[1], (20, true));
    }

    #[test]
    fn test_remove_empty_user_ids() {
        let db = MockDatabase::new();
        let group = Group { id: 1, default_notification_level: 2 };
        let manager = GroupManager::new(group, &db);

        let removed = manager.remove(&[]);
        assert!(removed.is_empty());
        assert!(db.removed_users.borrow().is_empty());
    }

    #[test]
    fn test_remove_users() {
        let db = MockDatabase::new();
        let group = Group { id: 1, default_notification_level: 2 };
        let manager = GroupManager::new(group, &db);

        let user_ids = vec![10, 20];
        let removed = manager.remove(&user_ids);

        assert_eq!(removed, user_ids);

        let removed_records = db.removed_users.borrow();
        assert_eq!(removed_records.len(), 1);
        assert_eq!(removed_records[0], (1, vec![10, 20]));

        let sync_remove = db.synced_removal_side_effects.borrow();
        assert_eq!(sync_remove.len(), 1);
        assert_eq!(sync_remove[0], (1, vec![10, 20]));

        let events = db.triggered_removed_events.borrow();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], 10);
        assert_eq!(events[1], 20);
    }
}
