#[derive(Debug, Clone, Default)]
pub struct OnCreateRecordConfig {
    pub rate_limits_disabled: bool,
    pub performed: bool,
}

impl OnCreateRecordConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn disable_rate_limits(&mut self) {
        self.rate_limits_disabled = true;
    }

    pub fn mark_performed(&mut self) {
        self.performed = true;
    }

    pub fn mark_rolled_back(&mut self) {
        self.performed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_create_record_config() {
        let mut config = OnCreateRecordConfig::new();
        assert!(!config.rate_limits_disabled);
        config.disable_rate_limits();
        assert!(config.rate_limits_disabled);
    }
}
