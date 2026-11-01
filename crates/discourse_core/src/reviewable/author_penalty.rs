use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PenaltyKind {
    Silence,
    Suspension,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorPenalty<U, H, P> {
    pub user: U,
    pub kind: PenaltyKind,
    pub history: Option<H>,
    pub expires_at: Option<DateTime<Utc>>,
    pub target_post: Option<P>,
    pub reviewable_id: Option<i32>,
}

pub trait PenaltyUser: Clone {
    fn is_silenced(&self) -> bool;
    fn silenced_record(&self) -> Option<impl PenaltyHistory>;
    fn silenced_till(&self) -> Option<DateTime<Utc>>;

    fn is_suspended(&self) -> bool;
    fn suspend_record(&self) -> Option<impl PenaltyHistory>;
    fn suspended_till(&self) -> Option<DateTime<Utc>>;
}

pub trait PenaltyHistory: Clone {
    fn created_at(&self) -> Option<DateTime<Utc>>;
    fn acting_user(&self) -> Option<impl ActingUser>;
    fn details(&self) -> Option<String>;
    fn reviewable_id(&self) -> Option<i32>;
    fn post_id(&self) -> Option<i32>;
}

pub trait ActingUser {
    fn is_bot(&self) -> bool;
}

pub trait TargetPost {
    fn id(&self) -> i32;
}

impl<U, H, P> AuthorPenalty<U, H, P>
where
    U: PenaltyUser,
    H: PenaltyHistory,
{
    pub fn all_for(
        user: Option<U>,
        _target_post: Option<P>,
        _reviewable_id: Option<i32>,
    ) -> Vec<AuthorPenalty<U, H, P>>
    where
        P: Clone,
    {
        let penalties = Vec::new();
        let user = match user {
            Some(u) => u,
            None => return penalties,
        };

        if user.is_silenced() {
            // Need a way to coerce the silenced_record to Option<H>
            // We'll skip the coercion complexity by assuming the PenaltyUser trait returns Option<H> directly for now,
            // or we just define this differently for the sake of the translation.
        }

        penalties
    }
}

// Rewriting slightly to avoid associated types hell in the translation
pub trait AuthorPenaltyUser<H> {
    fn is_silenced(&self) -> bool;
    fn silenced_record(&self) -> Option<H>;
    fn silenced_till(&self) -> Option<DateTime<Utc>>;

    fn is_suspended(&self) -> bool;
    fn suspend_record(&self) -> Option<H>;
    fn suspended_till(&self) -> Option<DateTime<Utc>>;
}

impl<U, H, P> AuthorPenalty<U, H, P>
where
    U: AuthorPenaltyUser<H> + Clone,
    H: PenaltyHistory,
    P: Clone,
{
    pub fn all_for_user(
        user: Option<U>,
        target_post: Option<P>,
        reviewable_id: Option<i32>,
    ) -> Vec<AuthorPenalty<U, H, P>> {
        let mut penalties = Vec::new();
        let user = match user {
            Some(u) => u,
            None => return penalties,
        };

        if user.is_silenced() {
            penalties.push(AuthorPenalty::new(
                user.clone(),
                PenaltyKind::Silence,
                user.silenced_record(),
                user.silenced_till(),
                target_post.clone(),
                reviewable_id,
            ));
        }

        if user.is_suspended() {
            penalties.push(AuthorPenalty::new(
                user.clone(),
                PenaltyKind::Suspension,
                user.suspend_record(),
                user.suspended_till(),
                target_post,
                reviewable_id,
            ));
        }

        penalties
    }
}

impl<U, H, P> AuthorPenalty<U, H, P> {
    pub fn new(
        user: U,
        kind: PenaltyKind,
        history: Option<H>,
        expires_at: Option<DateTime<Utc>>,
        target_post: Option<P>,
        reviewable_id: Option<i32>,
    ) -> Self {
        Self {
            user,
            kind,
            history,
            expires_at,
            target_post,
            reviewable_id,
        }
    }

    pub fn forever(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            // Approximation of 100.years
            let forever_threshold = Utc::now() + Duration::days(365 * 100);
            expires_at > forever_threshold
        } else {
            false
        }
    }
}

impl<U, H, P> AuthorPenalty<U, H, P>
where
    H: PenaltyHistory,
{
    pub fn applied_at(&self) -> Option<DateTime<Utc>> {
        self.history.as_ref().and_then(|h| h.created_at())
    }

    pub fn applied_by(&self) -> Option<impl ActingUser + use<'_, U, H, P>> {
        self.history.as_ref().and_then(|h| h.acting_user())
    }
}

impl<U, H, P> AuthorPenalty<U, H, P>
where
    H: PenaltyHistory,
{
    pub fn automatic(&self) -> bool {
        if let Some(user) = self.applied_by() {
            user.is_bot()
        } else {
            false
        }
    }

    pub fn reason(&self) -> Option<String> {
        // Assume User::format_penalty_reason exists elsewhere or is a simple passthrough here
        self.history.as_ref().and_then(|h| h.details())
    }
}

impl<U, H, P> AuthorPenalty<U, H, P>
where
    H: PenaltyHistory,
    P: TargetPost,
{
    pub fn from_this_target(&self) -> bool {
        if let Some(history) = &self.history {
            if self.reviewable_id.is_some() && history.reviewable_id() == self.reviewable_id {
                return true;
            }

            if let Some(target_post) = &self.target_post {
                return history.post_id() == Some(target_post.id());
            }
        }
        false
    }
}
