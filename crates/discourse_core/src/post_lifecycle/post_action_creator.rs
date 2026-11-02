use super::post_creator::{Post, Topic, User};

pub struct PostAction {
    pub id: i32,
    pub post_id: i32,
    pub user_id: i32,
    pub post_action_type_id: i32,
    pub staff_took_action: bool,
    pub targets_topic: bool,
}

pub struct ReviewableFlaggedPost {
    pub id: i32,
    pub created_by_id: i32,
    pub target_id: i32,
    pub potential_spam: bool,
    pub potentially_illegal: bool,
}

impl ReviewableFlaggedPost {
    pub fn needs_review(
        created_by: &User,
        target: &Post,
        _topic: &Topic,
        potential_spam: bool,
        potentially_illegal: bool,
    ) -> Self {
        Self {
            id: 100,
            created_by_id: created_by.id,
            target_id: target.id,
            potential_spam,
            potentially_illegal,
        }
    }
}

pub struct PostActionTypeView {
    pub notify_flag_type_ids: Vec<i32>,
    pub spam_type_id: i32,
    pub illegal_type_id: i32,
}

pub struct PostActionCreator<'a> {
    pub created_by: &'a User,
    pub post: &'a mut Post,
    pub topic: &'a mut Topic,
    pub post_action_type_id: i32,
    pub take_action: bool,
    pub post_action_type_view: PostActionTypeView,
    pub errors: Vec<String>,
}

pub struct Jobs;
impl Jobs {
    pub fn enqueue(job_name: &str, _post_id: i32) {
        println!("Enqueued job: {}", job_name);
    }
}

pub struct PostTransaction;
impl PostTransaction {
    pub fn execute<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        f()
    }
}

impl<'a> PostActionCreator<'a> {
    pub fn new(
        created_by: &'a User,
        post: &'a mut Post,
        topic: &'a mut Topic,
        post_action_type_id: i32,
        take_action: bool,
        post_action_type_view: PostActionTypeView,
    ) -> Self {
        Self {
            created_by,
            post,
            topic,
            post_action_type_id,
            take_action,
            post_action_type_view,
            errors: Vec::new(),
        }
    }

    pub fn perform(&mut self) -> Option<PostAction> {
        if self.post.hidden {
            self.errors.push("Cannot action a hidden post".to_string());
            return None;
        }

        let targets_topic = self.topic.posts_count != 1;

        let action = PostTransaction::execute(|| {
            PostAction {
                id: 1,
                post_id: self.post.id,
                user_id: self.created_by.id,
                post_action_type_id: self.post_action_type_id,
                staff_took_action: self.take_action,
                targets_topic,
            }
        });

        if self.take_action {
            self.post.hidden = true;
        }

        self.create_reviewable();

        Jobs::enqueue("process_post_action", self.post.id);

        Some(action)
    }

    pub fn create_reviewable(&self) -> Option<ReviewableFlaggedPost> {
        if !self
            .post_action_type_view
            .notify_flag_type_ids
            .contains(&self.post_action_type_id)
        {
            return None;
        }

        let is_bot_post = self.post.id < 0;
        if is_bot_post {
            return None;
        }

        let potential_spam = self.post_action_type_id == self.post_action_type_view.spam_type_id;
        let potentially_illegal =
            self.post_action_type_id == self.post_action_type_view.illegal_type_id;

        Some(ReviewableFlaggedPost::needs_review(
            self.created_by,
            self.post,
            self.topic,
            potential_spam,
            potentially_illegal,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_view() -> PostActionTypeView {
        PostActionTypeView {
            notify_flag_type_ids: vec![1, 2, 3],
            spam_type_id: 2,
            illegal_type_id: 3,
        }
    }

    #[test]
    fn test_perform_action_success() {
        let user = User { id: 1, username: "test".to_string(), staged: false };
        let mut post = Post { id: 10, topic_id: 5, raw: "hello".into(), hidden: false };
        let mut topic = Topic { id: 5, closed: false, private_message: false, posts_count: 5, closed_by_user_id: None, closed_message: None };

        let mut creator = PostActionCreator::new(&user, &mut post, &mut topic, 1, false, mock_view());
        let action = creator.perform().unwrap();

        assert_eq!(action.post_id, 10);
        assert_eq!(action.user_id, 1);
        assert!(!post.hidden);
    }

    #[test]
    fn test_perform_action_take_action() {
        let user = User { id: 1, username: "test".to_string(), staged: false };
        let mut post = Post { id: 10, topic_id: 5, raw: "hello".into(), hidden: false };
        let mut topic = Topic { id: 5, closed: false, private_message: false, posts_count: 5, closed_by_user_id: None, closed_message: None };

        let mut creator = PostActionCreator::new(&user, &mut post, &mut topic, 1, true, mock_view());
        let action = creator.perform().unwrap();

        assert!(action.staff_took_action);
        assert!(post.hidden);
    }

    #[test]
    fn test_perform_on_hidden_post_fails() {
        let user = User { id: 1, username: "test".to_string(), staged: false };
        let mut post = Post { id: 10, topic_id: 5, raw: "hello".into(), hidden: true };
        let mut topic = Topic { id: 5, closed: false, private_message: false, posts_count: 5, closed_by_user_id: None, closed_message: None };

        let mut creator = PostActionCreator::new(&user, &mut post, &mut topic, 1, false, mock_view());
        let action = creator.perform();

        assert!(action.is_none());
        assert_eq!(creator.errors[0], "Cannot action a hidden post");
    }

    #[test]
    fn test_create_reviewable_spam() {
        let user = User { id: 1, username: "test".to_string(), staged: false };
        let mut post = Post { id: 10, topic_id: 5, raw: "spam".into(), hidden: false };
        let mut topic = Topic { id: 5, closed: false, private_message: false, posts_count: 5, closed_by_user_id: None, closed_message: None };

        let creator = PostActionCreator::new(&user, &mut post, &mut topic, 2, false, mock_view());
        let reviewable = creator.create_reviewable().unwrap();

        assert_eq!(reviewable.created_by_id, 1);
        assert_eq!(reviewable.target_id, 10);
        assert!(reviewable.potential_spam);
        assert!(!reviewable.potentially_illegal);
    }

    #[test]
    fn test_create_reviewable_not_flag() {
        let user = User { id: 1, username: "test".to_string(), staged: false };
        let mut post = Post { id: 10, topic_id: 5, raw: "spam".into(), hidden: false };
        let mut topic = Topic { id: 5, closed: false, private_message: false, posts_count: 5, closed_by_user_id: None, closed_message: None };

        let creator = PostActionCreator::new(&user, &mut post, &mut topic, 4, false, mock_view());
        let reviewable = creator.create_reviewable();

        assert!(reviewable.is_none());
    }
}
