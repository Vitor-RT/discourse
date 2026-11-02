pub struct StylesheetManager {
    base_compiler_version: i32,
}

impl StylesheetManager {
    pub fn new() -> Self {
        Self {
            base_compiler_version: 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_initialization() {
        let manager = StylesheetManager::new();
        assert_eq!(manager.base_compiler_version, 7);
    }
}
