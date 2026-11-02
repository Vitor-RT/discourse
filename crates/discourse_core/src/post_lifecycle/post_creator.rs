pub struct User {
    pub id: i32,
    pub username: String,
    pub staged: bool,
}

pub struct Topic {
    pub id: i32,
    pub closed: bool,
    pub private_message: bool,
    pub posts_count: i32,
    pub closed_by_user_id: Option<i32>,
    pub closed_message: Option<String>,
}

impl Topic {
    pub fn update_status(&mut self, status: &str, value: bool, user_id: i32, message: &str) {
        if status == "closed" {
            self.closed = value;
            self.closed_by_user_id = Some(user_id);
            self.closed_message = Some(message.to_string());
        }
    }
}

pub struct Post {
    pub id: i32,
    pub topic_id: i32,
    pub raw: String,
    pub hidden: bool,
}

pub struct PostCreatorOpts {
    pub skip_validations: bool,
    pub raw: String,
}

pub struct PostCreator<'a> {
    pub user: &'a User,
    pub topic: &'a mut Topic,
    pub opts: PostCreatorOpts,
    pub errors: Vec<String>,
}

pub struct Jobs;
impl Jobs {
    pub fn enqueue(job_name: &str, _post_id: i32) {
        // mock sidekiq enqueue
        println!("Enqueued job: {}", job_name);
    }
}

pub struct PostTransaction;
impl PostTransaction {
    pub fn execute<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // mock ActiveRecord transaction
        f()
    }
}

impl<'a> PostCreator<'a> {
    pub fn new(user: &'a User, topic: &'a mut Topic, opts: PostCreatorOpts) -> Self {
        Self {
            user,
            topic,
            opts,
            errors: Vec::new(),
        }
    }

    pub fn create(&mut self) -> Option<Post> {
        if !self.opts.skip_validations && self.user.staged {
            self.errors.push("User is staged".to_string());
            return None;
        }

        if self.opts.raw.trim().is_empty() {
            self.errors.push("Raw content is empty".to_string());
            return None;
        }

        // Mock setup post
        let mut post = Post {
            id: 42,
            topic_id: self.topic.id,
            raw: self.opts.raw.clone(),
            hidden: false,
        };

        // Mock Transaction
        let saved = PostTransaction::execute(|| {
            // Mock save post
            if post.raw.contains("spam") {
                return false;
            }
            true
        });

        if !saved {
            self.errors.push("Failed to save post".to_string());
            return None;
        }

        // Mock enqueue sidekiq jobs
        Jobs::enqueue("process_post", post.id);

        self.auto_close(10, 50, -1);

        Some(post)
    }

    pub fn auto_close(
        &mut self,
        auto_close_messages_post_count: i32,
        auto_close_topics_post_count: i32,
        system_user_id: i32,
    ) {
        let is_private_message = self.topic.private_message;
        let topic_posts_count = self.topic.posts_count;

        if is_private_message
            && !self.topic.closed
            && auto_close_messages_post_count > 0
            && auto_close_messages_post_count <= topic_posts_count
        {
            self.topic.update_status(
                "closed",
                true,
                system_user_id,
                &format!(
                    "topic_statuses.autoclosed_message_max_posts count:{}",
                    auto_close_messages_post_count
                ),
            );
        } else if !is_private_message
            && !self.topic.closed
            && auto_close_topics_post_count > 0
            && auto_close_topics_post_count <= topic_posts_count
        {
            self.topic.update_status(
                "closed",
                true,
                system_user_id,
                &format!(
                    "topic_statuses.autoclosed_topic_max_posts count:{}",
                    auto_close_topics_post_count
                ),
            );

            Jobs::enqueue("create_linked_topic", 42);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_success() {
        let user = User { id: 1, username: "test".into(), staged: false };
        let mut topic = Topic { id: 1, closed: false, private_message: false, posts_count: 5, closed_by_user_id: None, closed_message: None };
        let opts = PostCreatorOpts { skip_validations: false, raw: "hello world".into() };

        let mut creator = PostCreator::new(&user, &mut topic, opts);
        let post = creator.create();

        assert!(post.is_some());
        assert_eq!(post.unwrap().raw, "hello world");
        assert!(creator.errors.is_empty());
    }

    #[test]
    fn test_create_staged_user() {
        let user = User { id: 1, username: "test".into(), staged: true };
        let mut topic = Topic { id: 1, closed: false, private_message: false, posts_count: 5, closed_by_user_id: None, closed_message: None };
        let opts = PostCreatorOpts { skip_validations: false, raw: "hello world".into() };

        let mut creator = PostCreator::new(&user, &mut topic, opts);
        let post = creator.create();

        assert!(post.is_none());
        assert_eq!(creator.errors[0], "User is staged");
    }

    #[test]
    fn test_create_spam_fails() {
        let user = User { id: 1, username: "test".into(), staged: false };
        let mut topic = Topic { id: 1, closed: false, private_message: false, posts_count: 5, closed_by_user_id: None, closed_message: None };
        let opts = PostCreatorOpts { skip_validations: false, raw: "this is spam".into() };

        let mut creator = PostCreator::new(&user, &mut topic, opts);
        let post = creator.create();

        assert!(post.is_none());
        assert_eq!(creator.errors[0], "Failed to save post");
    }
}
