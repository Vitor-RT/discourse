use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
}

impl Item {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

pub trait CollectionItem {
    fn server_action(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct Collection<T, R, G, A> {
    pub reviewable: R,
    pub guardian: G,
    pub args: Option<A>,
    pub content: Vec<T>,
}

impl<T, R, G, A> Collection<T, R, G, A> {
    pub fn new(reviewable: R, guardian: G, args: Option<A>) -> Self {
        Self {
            reviewable,
            guardian,
            args,
            content: Vec::new(),
        }
    }

    pub fn is_present(&self) -> bool {
        !self.content.is_empty()
    }

    pub fn is_blank(&self) -> bool {
        self.content.is_empty()
    }

    pub fn to_a(&self) -> &Vec<T> {
        &self.content
    }
}

impl<T, R, G, A> Collection<T, R, G, A>
where
    T: CollectionItem,
{
    pub fn has(&self, action_id: &str) -> bool {
        self.content.iter().any(|a| a.server_action() == action_id)
    }
}
