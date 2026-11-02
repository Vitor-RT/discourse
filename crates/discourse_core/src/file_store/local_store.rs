pub struct LocalStore;

impl LocalStore {
    pub fn external() -> bool {
        false
    }

    pub fn is_relative(url: &str, relative_base_url: &str) -> bool {
        !url.is_empty() && url.starts_with(relative_base_url)
    }

    pub fn tombstone_dir(public_dir: &str, relative_base_url: &str) -> String {
        format!(
            "{}{}",
            public_dir,
            relative_base_url.replace("/uploads/", "/uploads/tombstone/")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_store() {
        assert!(!LocalStore::external());
        assert!(LocalStore::is_relative(
            "/uploads/default/1.png",
            "/uploads/"
        ));
    }
}
