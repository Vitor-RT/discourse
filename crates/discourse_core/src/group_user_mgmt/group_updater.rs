use std::collections::HashMap;

pub trait GroupUpdaterDatabase {
    fn update_group_attributes(&self, group_id: i32, attributes: &HashMap<String, String>) -> bool;
    fn can_admin_group(&self, user_id: i32, group_id: i32) -> bool;
    fn can_use_flair_upload(&self, user_id: i32, upload_id: i32) -> bool;
    fn update_users_notification_level(&self, group_id: i32, old_level: i32, new_level: i32);
    fn log_change_group_settings(&self, user_id: i32, group_id: i32);
}

pub struct Guardian {
    pub user_id: i32,
    pub is_staff: bool,
    pub is_admin: bool,
}

#[derive(Clone)]
pub struct GroupRecord {
    pub id: i32,
    pub automatic: bool,
    pub smtp_enabled: bool,
    pub flair_upload_id: Option<i32>,
    pub default_notification_level: i32,
}

pub struct GroupUpdater<'a, D: GroupUpdaterDatabase> {
    guardian: Guardian,
    group: GroupRecord,
    attributes: HashMap<String, String>,
    update_existing_users: Option<bool>,
    db: &'a D,
}

impl<'a, D: GroupUpdaterDatabase> GroupUpdater<'a, D> {
    pub fn new(
        guardian: Guardian,
        group: GroupRecord,
        attributes: HashMap<String, String>,
        update_existing_users: Option<bool>,
        db: &'a D,
    ) -> Self {
        Self {
            guardian,
            group,
            attributes,
            update_existing_users,
            db,
        }
    }

    fn valid_flair_upload(&self, group_attributes: &HashMap<String, String>) -> bool {
        if !group_attributes.contains_key("flair_upload_id") {
            return true;
        }

        if let Some(upload_id_str) = group_attributes.get("flair_upload_id") {
            if upload_id_str.is_empty() {
                return true;
            }
            if let Ok(upload_id) = upload_id_str.parse::<i32>() {
                if let Some(group_flair_id) = self.group.flair_upload_id {
                    if upload_id == group_flair_id {
                        return true;
                    }
                }
                return self.db.can_use_flair_upload(self.guardian.user_id, upload_id);
            }
        }

        true
    }

    pub fn update(&self) -> Result<bool, String> {
        let is_admin = self.db.can_admin_group(self.guardian.user_id, self.group.id);
        if !is_admin && !self.guardian.is_staff {
            return Err("Cannot edit group".to_string());
        }

        let mut group_attributes = self.attributes.clone();

        if self.group.smtp_enabled && group_attributes.get("smtp_enabled").map(|s| s.as_str()) == Some("false") {
            group_attributes.insert("smtp_server".to_string(), "".to_string());
            group_attributes.insert("smtp_port".to_string(), "".to_string());
            group_attributes.insert("email_username".to_string(), "".to_string());
            group_attributes.insert("email_password".to_string(), "".to_string());
        }

        if !self.valid_flair_upload(&group_attributes) {
            return Ok(false);
        }

        let mut old_level = None;
        let mut new_level = None;

        if let Some(default_level_str) = group_attributes.get("default_notification_level") {
            if let Ok(default_level) = default_level_str.parse::<i32>() {
                if self.group.default_notification_level != default_level {
                    old_level = Some(self.group.default_notification_level);
                    new_level = Some(default_level);
                }
            }
        }

        if old_level.is_some() && self.update_existing_users.is_none() {
            return Err("Existing users confirmation required".to_string());
        }

        if !self.db.update_group_attributes(self.group.id, &group_attributes) {
            return Ok(false);
        }

        self.db.log_change_group_settings(self.guardian.user_id, self.group.id);

        if self.update_existing_users == Some(true) {
            if let (Some(old), Some(new)) = (old_level, new_level) {
                self.db.update_users_notification_level(self.group.id, old, new);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockDb {
        updated_group_attributes: RefCell<Vec<(i32, HashMap<String, String>)>>,
        updated_users_notification_level: RefCell<Vec<(i32, i32, i32)>>,
        logged_changes: RefCell<Vec<(i32, i32)>>,
        is_admin: bool,
        can_use_flair: bool,
    }

    impl MockDb {
        fn new(is_admin: bool, can_use_flair: bool) -> Self {
            Self {
                updated_group_attributes: RefCell::new(vec![]),
                updated_users_notification_level: RefCell::new(vec![]),
                logged_changes: RefCell::new(vec![]),
                is_admin,
                can_use_flair,
            }
        }
    }

    impl GroupUpdaterDatabase for MockDb {
        fn update_group_attributes(&self, group_id: i32, attributes: &HashMap<String, String>) -> bool {
            self.updated_group_attributes.borrow_mut().push((group_id, attributes.clone()));
            true
        }

        fn can_admin_group(&self, _user_id: i32, _group_id: i32) -> bool {
            self.is_admin
        }

        fn can_use_flair_upload(&self, _user_id: i32, _upload_id: i32) -> bool {
            self.can_use_flair
        }

        fn update_users_notification_level(&self, group_id: i32, old_level: i32, new_level: i32) {
            self.updated_users_notification_level.borrow_mut().push((group_id, old_level, new_level));
        }

        fn log_change_group_settings(&self, user_id: i32, group_id: i32) {
            self.logged_changes.borrow_mut().push((user_id, group_id));
        }
    }

    #[test]
    fn test_unauthorized_user() {
        let db = MockDb::new(false, false);
        let guardian = Guardian { user_id: 1, is_staff: false, is_admin: false };
        let group = GroupRecord { id: 1, automatic: false, smtp_enabled: false, flair_upload_id: None, default_notification_level: 1 };

        let updater = GroupUpdater::new(guardian, group, HashMap::new(), None, &db);
        let res = updater.update();

        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Cannot edit group");
    }

    #[test]
    fn test_invalid_flair_upload() {
        let db = MockDb::new(true, false); // Admin but can't use flair
        let guardian = Guardian { user_id: 1, is_staff: true, is_admin: true };
        let group = GroupRecord { id: 1, automatic: false, smtp_enabled: false, flair_upload_id: Some(10), default_notification_level: 1 };

        let mut attrs = HashMap::new();
        attrs.insert("flair_upload_id".to_string(), "20".to_string()); // Different upload id

        let updater = GroupUpdater::new(guardian, group, attrs, None, &db);
        let res = updater.update();

        assert_eq!(res, Ok(false)); // Should fail silently by returning false
    }

    #[test]
    fn test_successful_update() {
        let db = MockDb::new(true, true);
        let guardian = Guardian { user_id: 1, is_staff: true, is_admin: true };
        let group = GroupRecord { id: 2, automatic: false, smtp_enabled: false, flair_upload_id: None, default_notification_level: 1 };

        let mut attrs = HashMap::new();
        attrs.insert("name".to_string(), "New Name".to_string());

        let updater = GroupUpdater::new(guardian, group, attrs, None, &db);
        let res = updater.update();

        assert_eq!(res, Ok(true));
        assert_eq!(db.updated_group_attributes.borrow().len(), 1);
        assert_eq!(db.logged_changes.borrow().len(), 1);
    }
}
