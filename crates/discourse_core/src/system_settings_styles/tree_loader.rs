pub struct TreeLoader {
    pub topic_id: i32,
}

impl TreeLoader {
    pub fn new(topic_id: i32) -> Self {
        Self { topic_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_loader_initialization() {
        let loader = TreeLoader::new(1);
        assert_eq!(loader.topic_id, 1);
    }
}
