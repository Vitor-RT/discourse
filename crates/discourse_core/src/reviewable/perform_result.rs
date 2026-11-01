use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Success,
    Failure, // Added common alternative to success
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformResult<R, P, T, U> {
    pub reviewable: R,
    pub status: Status,
    pub created_post: Option<P>,
    pub created_post_topic: Option<T>,

    pub transition_to: Option<String>,
    pub remove_reviewable_ids: Vec<i32>,
    pub affected_reviewable_ids: Vec<i32>,
    pub errors: Option<Vec<String>>, // Simplified error type
    pub recalculate_score: Option<bool>,
    pub update_flag_stats: Option<bool>,
    pub after_commit: Option<String>, // Placeholder for arbitrary callback
    pub post_revision: Option<bool>,
    pub deleted_user: Option<U>,
    pub removed_avatar: Option<bool>,
}

pub trait ReviewableEntity {
    fn id(&self) -> i32;
}

pub trait PostEntity<T> {
    fn topic(&self) -> Option<T>;
}

impl<R, P, T, U> PerformResult<R, P, T, U>
where
    R: ReviewableEntity,
{
    pub fn new(reviewable: R, status: Status) -> Self {
        let remove_reviewable_ids = if status == Status::Success {
            vec![reviewable.id()]
        } else {
            Vec::new()
        };

        Self {
            reviewable,
            status,
            created_post: None,
            created_post_topic: None,
            transition_to: None,
            remove_reviewable_ids,
            affected_reviewable_ids: Vec::new(),
            errors: None,
            recalculate_score: None,
            update_flag_stats: None,
            after_commit: None,
            post_revision: None,
            deleted_user: None,
            removed_avatar: None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.status == Status::Success
    }
}

impl<R, P, T, U> PerformResult<R, P, T, U>
where
    P: PostEntity<T> + Clone,
{
    pub fn set_created_post(&mut self, created_post: P) {
        self.created_post_topic = created_post.topic();
        self.created_post = Some(created_post);
    }
}
