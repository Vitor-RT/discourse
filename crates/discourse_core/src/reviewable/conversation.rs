use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post<U> {
    pub id: i32,
    pub user: Option<U>,
    pub excerpt: String,
}

impl<U> Post<U> {
    pub fn new(id: i32, user: Option<U>, excerpt: String) -> Self {
        Self { id, user, excerpt }
    }

    pub fn excerpt(cooked: &str) -> String {
        // Placeholder for `Post.excerpt(cooked, 250, keep_emoji_images: true)`
        // Simplified excerpt generation logic for Rust
        let fragment = Html::parse_fragment(cooked);

        // Remove the first link if it's the first node
        // (Assuming a specific HTML structure, this is simplified)
        let _a_selector = Selector::parse("a:first-child").unwrap();
        let mut html_out = String::new();

        let mut first = true;
        for node in fragment.tree.root().children() {
            if let Some(element) = node.value().as_element() {
                if first && element.name() == "a" {
                    first = false;
                    continue; // Skip it
                }
            }
            first = false;
            // A more robust implementation would serialize the node back to HTML.
            // For now, we will just use the text.
            if let Some(text) = node.value().as_text() {
                html_out.push_str(&text.text);
            }
        }

        let excerpt = html_out.trim().to_string();

        if excerpt.len() > 250 {
            format!("{}...", &excerpt[..247])
        } else {
            excerpt
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation<U> {
    pub id: i32,
    pub permalink: String,
    pub has_more: bool,
    pub conversation_posts: Vec<Post<U>>,
}

pub trait MetaTopic<U> {
    fn id(&self) -> i32;
    fn relative_url(&self) -> String;
    fn posts_count(&self) -> i32;
    // ordered_posts filtering post_type: regular and limit: 2
    fn get_conversation_posts(&self) -> Vec<impl MetaPost<U>>;
}

pub trait MetaPost<U> {
    fn id(&self) -> i32;
    fn user(&self) -> Option<U>;
    fn cooked(&self) -> String;
}

impl<U> Conversation<U>
where
    U: Clone,
{
    pub fn new(meta_topic: &impl MetaTopic<U>, base_url: &str) -> Self {
        let permalink = format!("{}{}", base_url, meta_topic.relative_url());

        let meta_posts = meta_topic.get_conversation_posts();

        let conversation_posts: Vec<Post<U>> = meta_posts
            .iter()
            .map(|mp| Post::new(mp.id(), mp.user(), Post::<U>::excerpt(&mp.cooked())))
            .collect();

        let has_more = meta_topic.posts_count() > 2;

        Self {
            id: meta_topic.id(),
            permalink,
            has_more,
            conversation_posts,
        }
    }
}
