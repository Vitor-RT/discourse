use std::collections::{HashMap, HashSet};

pub trait UserCommDatabase {
    fn is_staff(&self, user_id: i32) -> bool;
    fn get_user_option_allow_private_messages(&self, user_id: i32) -> bool;
    fn get_user_option_enable_allowed_pm_users(&self, user_id: i32) -> bool;
    fn is_user_muting(&self, actor_id: i32, target_id: i32) -> bool;
    fn is_user_ignoring(&self, actor_id: i32, target_id: i32) -> bool;
    fn is_in_allowed_pm_users(&self, actor_id: i32, target_id: i32) -> bool;
}

#[derive(Clone, Debug)]
pub struct UserCommPref {
    pub user_id: i32,
    pub is_muting: bool,
    pub is_ignoring: bool,
    pub is_disallowing_all_pms: bool,
    pub is_disallowing_pms_from_acting_user: bool,
}

impl UserCommPref {
    pub fn communication_prevented(&self) -> bool {
        self.ignoring_or_muting() || self.disallowing_pms()
    }

    pub fn ignoring_or_muting(&self) -> bool {
        self.is_muting || self.is_ignoring
    }

    pub fn disallowing_pms(&self) -> bool {
        self.is_disallowing_all_pms || self.is_disallowing_pms_from_acting_user
    }
}

pub struct UserCommScreener<'a, D: UserCommDatabase> {
    pub acting_user_id: i32,
    pub target_user_ids: Vec<i32>,
    db: &'a D,
    preferences: HashMap<i32, UserCommPref>,
}

impl<'a, D: UserCommDatabase> UserCommScreener<'a, D> {
    pub fn new(acting_user_id: i32, target_user_ids: Vec<i32>, db: &'a D) -> Self {
        let target_user_ids: Vec<i32> = target_user_ids.into_iter().filter(|&id| id != acting_user_id).collect();
        let mut screener = Self {
            acting_user_id,
            target_user_ids,
            db,
            preferences: HashMap::new(),
        };
        screener.load_preference_map();
        screener
    }

    fn load_preference_map(&mut self) {
        if self.db.is_staff(self.acting_user_id) {
            return;
        }

        for &user_id in &self.target_user_ids {
            let allow_pms = self.db.get_user_option_allow_private_messages(user_id);
            let enable_allowed_pm_users = self.db.get_user_option_enable_allowed_pm_users(user_id);
            let is_muting = self.db.is_user_muting(self.acting_user_id, user_id);
            let is_ignoring = self.db.is_user_ignoring(self.acting_user_id, user_id);

            let mut is_disallowing_pms_from_acting_user = false;

            if enable_allowed_pm_users {
                if !self.db.is_in_allowed_pm_users(self.acting_user_id, user_id) {
                    is_disallowing_pms_from_acting_user = true;
                }
            }

            if !allow_pms || enable_allowed_pm_users || is_muting || is_ignoring {
                self.preferences.insert(user_id, UserCommPref {
                    user_id,
                    is_muting,
                    is_ignoring,
                    is_disallowing_all_pms: !allow_pms,
                    is_disallowing_pms_from_acting_user,
                });
            }
        }
    }

    pub fn allowing_actor_communication(&self) -> Vec<i32> {
        let mut allowed: HashSet<i32> = self.target_user_ids.iter().cloned().collect();

        if !self.db.is_staff(self.acting_user_id) {
            for (user_id, pref) in &self.preferences {
                if pref.communication_prevented() {
                    allowed.remove(user_id);
                }
            }
        }

        let mut result: Vec<i32> = allowed.into_iter().collect();
        result.sort_unstable();
        result
    }

    pub fn preventing_actor_communication(&self) -> Vec<i32> {
        if self.db.is_staff(self.acting_user_id) {
            return vec![];
        }

        let mut prevented: Vec<i32> = self.preferences.values()
            .filter(|pref| pref.communication_prevented())
            .map(|pref| pref.user_id)
            .collect();
        prevented.sort_unstable();
        prevented
    }

    pub fn ignoring_or_muting_actor(&self, user_id: i32) -> bool {
        if self.acting_user_id == user_id || self.db.is_staff(self.acting_user_id) {
            return false;
        }

        self.preferences.get(&user_id)
            .map(|pref| pref.ignoring_or_muting())
            .unwrap_or(false)
    }

    pub fn disallowing_pms_from_actor(&self, user_id: i32) -> bool {
        if self.acting_user_id == user_id || self.db.is_staff(self.acting_user_id) {
            return false;
        }

        self.preferences.get(&user_id)
            .map(|pref| pref.disallowing_pms() || pref.ignoring_or_muting())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDb {
        is_staff: bool,
        allow_pms: bool,
        enable_allowed: bool,
        muting: bool,
        ignoring: bool,
        in_allowed: bool,
    }

    impl UserCommDatabase for MockDb {
        fn is_staff(&self, _user_id: i32) -> bool { self.is_staff }
        fn get_user_option_allow_private_messages(&self, _user_id: i32) -> bool { self.allow_pms }
        fn get_user_option_enable_allowed_pm_users(&self, _user_id: i32) -> bool { self.enable_allowed }
        fn is_user_muting(&self, _actor_id: i32, _target_id: i32) -> bool { self.muting }
        fn is_user_ignoring(&self, _actor_id: i32, _target_id: i32) -> bool { self.ignoring }
        fn is_in_allowed_pm_users(&self, _actor_id: i32, _target_id: i32) -> bool { self.in_allowed }
    }

    #[test]
    fn test_staff_actor() {
        let db = MockDb {
            is_staff: true,
            allow_pms: false,
            enable_allowed: false,
            muting: true,
            ignoring: true,
            in_allowed: false,
        };
        let screener = UserCommScreener::new(1, vec![2, 3], &db);

        assert_eq!(screener.allowing_actor_communication(), vec![2, 3]);
        assert!(screener.preventing_actor_communication().is_empty());
        assert!(!screener.ignoring_or_muting_actor(2));
        assert!(!screener.disallowing_pms_from_actor(2));
    }

    #[test]
    fn test_non_staff_muting() {
        let db = MockDb {
            is_staff: false,
            allow_pms: true,
            enable_allowed: false,
            muting: true,
            ignoring: false,
            in_allowed: false,
        };
        let screener = UserCommScreener::new(1, vec![2], &db);

        assert!(screener.allowing_actor_communication().is_empty());
        assert_eq!(screener.preventing_actor_communication(), vec![2]);
        assert!(screener.ignoring_or_muting_actor(2));
        assert!(screener.disallowing_pms_from_actor(2)); // muting implies disallowing
    }

    #[test]
    fn test_disallow_all_pms() {
        let db = MockDb {
            is_staff: false,
            allow_pms: false,
            enable_allowed: false,
            muting: false,
            ignoring: false,
            in_allowed: false,
        };
        let screener = UserCommScreener::new(1, vec![2], &db);

        assert!(screener.allowing_actor_communication().is_empty());
        assert_eq!(screener.preventing_actor_communication(), vec![2]);
        assert!(!screener.ignoring_or_muting_actor(2));
        assert!(screener.disallowing_pms_from_actor(2));
    }

    #[test]
    fn test_allow_pms_with_whitelist() {
        let db = MockDb {
            is_staff: false,
            allow_pms: true, // Though typically this combines, we test logic flow
            enable_allowed: true,
            muting: false,
            ignoring: false,
            in_allowed: true,
        };
        let screener = UserCommScreener::new(1, vec![2], &db);

        // the actor is in the whitelist, communication allowed
        assert_eq!(screener.allowing_actor_communication(), vec![2]);
        assert!(screener.preventing_actor_communication().is_empty());
        assert!(!screener.ignoring_or_muting_actor(2));
        assert!(!screener.disallowing_pms_from_actor(2));
    }

    #[test]
    fn test_allow_pms_not_in_whitelist() {
        let db = MockDb {
            is_staff: false,
            allow_pms: true,
            enable_allowed: true,
            muting: false,
            ignoring: false,
            in_allowed: false,
        };
        let screener = UserCommScreener::new(1, vec![2], &db);

        // actor not in whitelist, communication prevented
        assert!(screener.allowing_actor_communication().is_empty());
        assert_eq!(screener.preventing_actor_communication(), vec![2]);
        assert!(!screener.ignoring_or_muting_actor(2));
        assert!(screener.disallowing_pms_from_actor(2));
    }
}
