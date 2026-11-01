use super::collection::{Collection, CollectionItem, Item};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Bundle {
    pub item: Item,
    pub icon: Option<String>,
    pub label: Option<String>,
    pub secondary: bool,
    pub actions: Vec<Action>,
}

impl Bundle {
    pub fn new(id: String, icon: Option<String>, label: Option<String>, secondary: bool) -> Self {
        Self {
            item: Item::new(id),
            icon,
            label,
            secondary,
            actions: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn bundle_id(&self) -> String {
        let parts: Vec<&str> = self.item.id.splitn(2, '-').collect();
        if parts.len() > 1 {
            parts[1].to_string()
        } else {
            self.item.id.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Action {
    pub item: Item,
    pub icon: Option<String>,
    pub button_class: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub confirm_message: Option<String>,
    pub confirm_message_args: Option<HashMap<String, String>>,
    pub confirm_destructive: Option<bool>,
    pub client_action: Option<String>,
    pub require_reject_reason: Option<bool>,
    pub custom_modal: Option<String>,
    pub completed_message: Option<String>,
    pub penalty_effect: Option<String>,
}

impl Action {
    pub fn new(
        id: String,
        icon: Option<String>,
        button_class: Option<String>,
        label: Option<String>,
    ) -> Self {
        Self {
            item: Item::new(id),
            icon,
            button_class,
            label,
            description: None,
            confirm_message: None,
            confirm_message_args: None,
            confirm_destructive: None,
            client_action: None,
            require_reject_reason: None,
            custom_modal: None,
            completed_message: None,
            penalty_effect: None,
        }
    }

    pub fn action_name(&self) -> String {
        let parts: Vec<&str> = self.item.id.splitn(2, '-').collect();
        if parts.len() > 1 {
            parts[1].to_string()
        } else {
            self.item.id.clone()
        }
    }
}

impl CollectionItem for Action {
    fn server_action(&self) -> String {
        let parts: Vec<&str> = self.item.id.split('-').collect();
        parts.last().unwrap_or(&self.item.id.as_str()).to_string()
    }
}

pub trait ActionsReviewable {
    fn target_type(&self) -> Option<String>;
    fn id(&self) -> i32;
}

#[derive(Debug, Clone)]
pub struct Actions<R, G, A> {
    pub collection: Collection<Action, R, G, A>,
    pub bundles: Vec<Bundle>,
}

impl<R, G, A> Actions<R, G, A>
where
    R: ActionsReviewable + Clone,
{
    pub fn new(reviewable: R, guardian: G, args: Option<A>) -> Self {
        Self {
            collection: Collection::new(reviewable, guardian, args),
            bundles: Vec::new(),
        }
    }

    pub fn common_actions() -> HashMap<String, Action> {
        let mut actions = HashMap::new();
        actions.insert(
            "approve".to_string(),
            Action::new(
                "approve".to_string(),
                Some("thumbs-up".to_string()),
                None,
                Some("reviewables.actions.approve.title".to_string()),
            ),
        );
        actions.insert(
            "reject".to_string(),
            Action::new(
                "reject".to_string(),
                Some("thumbs-down".to_string()),
                None,
                Some("reviewables.actions.reject.title".to_string()),
            ),
        );
        actions.insert(
            "delete".to_string(),
            Action::new(
                "delete".to_string(),
                Some("trash-can".to_string()),
                None,
                Some("reviewables.actions.delete.title".to_string()),
            ),
        );
        actions
    }

    pub fn add_bundle(
        &mut self,
        id: String,
        icon: Option<String>,
        label: Option<String>,
        secondary: bool,
    ) -> usize {
        let bundle = Bundle::new(id, icon, label, secondary);
        self.bundles.push(bundle);
        self.bundles.len() - 1
    }

    pub fn add<F>(
        &mut self,
        id: String,
        bundle_index: Option<usize>,
        secondary: bool,
        mut callback: F,
    ) where
        F: FnMut(&mut Action),
    {
        // Compute action name
        let target_type = self
            .collection
            .reviewable
            .target_type()
            .unwrap_or_default()
            .to_lowercase()
            .replace("::", "_");

        let action_name = if target_type.is_empty() {
            id.clone()
        } else if id.is_empty() {
            target_type
        } else {
            format!("{}-{}", target_type, id)
        };

        // Compute scoped id
        let reviewable_id = self.collection.reviewable.id().to_string();
        let scoped_id = if reviewable_id.is_empty() {
            action_name.clone()
        } else if action_name.is_empty() {
            reviewable_id
        } else {
            format!("{}-{}", reviewable_id, action_name)
        };

        let mut common = Self::common_actions();
        let mut action = if let Some(common_action) = common.remove(&action_name) {
            common_action
        } else {
            Action::new(scoped_id.clone(), None, None, None)
        };

        callback(&mut action);

        self.collection.content.push(action.clone());

        let idx = if let Some(idx) = bundle_index {
            idx
        } else {
            self.add_bundle(scoped_id, None, None, secondary)
        };

        if let Some(bundle) = self.bundles.get_mut(idx) {
            bundle.actions.push(action);
        }
    }
}
