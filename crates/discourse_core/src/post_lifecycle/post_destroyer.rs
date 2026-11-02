use super::post_creator::{Post, Topic, User};

pub struct Guardian<'a> {
    pub user: &'a User,
}

impl<'a> Guardian<'a> {
    pub fn new(user: &'a User) -> Self {
        Self { user }
    }

    pub fn can_moderate_topic(&self, _topic: &Topic) -> bool {
        self.user.username == "moderator" || self.user.username == "admin"
    }

    pub fn can_delete_all_posts_and_topics(&self) -> bool {
        self.user.username == "admin"
    }

    pub fn can_review_topic(&self, _topic: &Topic) -> bool {
        self.user.username != "banned_user"
    }
}

pub struct Reviewable {
    pub target_id: i32,
}

impl Reviewable {
    pub fn exists(target: &Post) -> bool {
        target.id == 42 // mock arbitrary logic for reviewable post
    }
}

#[derive(Default)]
pub struct PostDestroyerOpts {
    pub force_destroy: bool,
    pub permanent: bool,
    pub defer_flags: bool,
    pub reviewable_id: Option<i32>,
}

pub struct PostDestroyer<'a> {
    pub user: &'a User,
    pub topic: &'a mut Topic,
    pub post: &'a mut Post,
    pub opts: PostDestroyerOpts,
    guardian: Guardian<'a>,
}

pub struct Jobs;
impl Jobs {
    pub fn enqueue(job_name: &str, _post_id: i32) {
        println!("Enqueued job: {}", job_name);
    }
}

impl<'a> PostDestroyer<'a> {
    pub fn new(
        user: &'a User,
        topic: &'a mut Topic,
        post: &'a mut Post,
        opts: PostDestroyerOpts,
    ) -> Self {
        let guardian = Guardian::new(user);
        Self {
            user,
            topic,
            post,
            opts,
            guardian,
        }
    }

    pub fn destroy(&mut self) -> bool {
        if !self.guardian.can_delete_all_posts_and_topics() && !self.opts.force_destroy {
            return false;
        }

        if self.opts.permanent {
            // "destroy!"
            println!("Post permanently deleted from DB");
            self.post.hidden = true;
            return true;
        }

        // Soft delete (hide)
        self.post.hidden = true;

        self.topic.posts_count -= 1;
        self.make_previous_post_the_last_one();

        Jobs::enqueue("feature_topic_users", self.topic.id);

        true
    }

    pub fn recover(&mut self) -> bool {
        if !self.post.hidden {
            return false;
        }

        self.post.hidden = false;
        self.topic.posts_count += 1;
        self.make_previous_post_the_last_one();

        Jobs::enqueue("feature_topic_users", self.topic.id);

        true
    }

    fn make_previous_post_the_last_one(&self) {
        // mock logic to fetch previous post and update topic
        println!("Topic updated with previous post stats");
    }

    pub fn can_moderate_this_topic(&self) -> bool {
        self.guardian.can_moderate_topic(self.topic)
            || self.guardian.can_delete_all_posts_and_topics()
    }

    pub fn post_is_reviewable(&self, is_staff: bool) -> bool {
        if is_staff {
            return true;
        }

        self.guardian.can_review_topic(self.topic) && Reviewable::exists(self.post)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destroy_post_success() {
        let user = User { id: 1, username: "admin".to_string(), staged: false };
        let mut topic = Topic { id: 1, closed: false, private_message: false, posts_count: 5, closed_by_user_id: None, closed_message: None };
        let mut post = Post { id: 1, topic_id: 1, raw: "bad post".into(), hidden: false };

        let mut destroyer = PostDestroyer::new(&user, &mut topic, &mut post, PostDestroyerOpts::default());
        assert!(destroyer.destroy());

        assert!(post.hidden);
        assert_eq!(topic.posts_count, 4);
    }

    #[test]
    fn test_destroy_post_permission_denied() {
        let user = User { id: 2, username: "user".to_string(), staged: false };
        let mut topic = Topic { id: 1, closed: false, private_message: false, posts_count: 5, closed_by_user_id: None, closed_message: None };
        let mut post = Post { id: 1, topic_id: 1, raw: "bad post".into(), hidden: false };

        let mut destroyer = PostDestroyer::new(&user, &mut topic, &mut post, PostDestroyerOpts::default());
        assert!(!destroyer.destroy());
        assert!(!post.hidden);
    }

    #[test]
    fn test_recover_post() {
        let user = User { id: 1, username: "admin".to_string(), staged: false };
        let mut topic = Topic { id: 1, closed: false, private_message: false, posts_count: 4, closed_by_user_id: None, closed_message: None };
        let mut post = Post { id: 1, topic_id: 1, raw: "good post".into(), hidden: true };

        let mut destroyer = PostDestroyer::new(&user, &mut topic, &mut post, PostDestroyerOpts::default());
        assert!(destroyer.recover());

        assert!(!post.hidden);
        assert_eq!(topic.posts_count, 5);
    }
}
