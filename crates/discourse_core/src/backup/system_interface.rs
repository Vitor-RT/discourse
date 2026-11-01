use crate::backup::logger::Logger;

pub struct SystemInterface<'a> {
    logger: &'a mut Logger,
    readonly_mode_was_enabled: bool,
    current_db: String,
}

impl<'a> SystemInterface<'a> {
    pub fn new(logger: &'a mut Logger) -> Self {
        let current_db = "default".to_string();
        let readonly_mode_was_enabled = false;

        Self {
            logger,
            readonly_mode_was_enabled,
            current_db,
        }
    }

    pub fn enable_readonly_mode(&mut self) {
        if self.readonly_mode_was_enabled {
            return;
        }
        self.logger.log("Enabling readonly mode...", None);
    }

    pub fn disable_readonly_mode(&mut self) {
        if self.readonly_mode_was_enabled {
            return;
        }
        self.logger.log("Disabling readonly mode...", None);
    }

    pub fn mark_restore_as_running(&mut self) {
        self.logger.log("Marking restore as running...", None);
    }

    pub fn mark_restore_as_not_running(&mut self) {
        self.logger.log("Marking restore as finished...", None);
    }

    pub fn listen_for_shutdown_signal(&mut self) {}

    pub fn pause_sidekiq(&mut self, _reason: &str) {
        self.logger.log("Pausing sidekiq...", None);
    }

    pub fn unpause_sidekiq(&mut self) {
        self.logger.log("Unpausing sidekiq...", None);
    }

    pub fn wait_for_sidekiq(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let max_wait_seconds = 60;
        let wait_seconds = 6;

        self.logger.log(
            &format!(
                "Waiting up to {} seconds for Sidekiq to finish running jobs...",
                max_wait_seconds
            ),
            None,
        );

        let max_iterations = max_wait_seconds / wait_seconds;
        let mut iterations = 1;

        loop {
            if !self.sidekiq_has_running_jobs() {
                break;
            }

            iterations += 1;
            if iterations > max_iterations {
                return Err(
                    "Sidekiq did not finish running all the jobs in the allowed time!".into(),
                );
            }

            self.logger.log(
                &format!(
                    "Waiting for sidekiq to finish running jobs... #{}",
                    iterations
                ),
                None,
            );
        }

        Ok(())
    }

    pub fn flush_redis(&mut self) {}

    pub fn clear_sidekiq_queues(&mut self) {}

    fn sidekiq_has_running_jobs(&self) -> bool {
        false
    }

    fn _delete_job_if_it_belongs_to_current_site(&self, _job: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::logger::Logger;

    #[test]
    fn test_system_interface_initialization() {
        let mut logger = Logger::new(None, None);
        let interface = SystemInterface::new(&mut logger);
        assert_eq!(interface.current_db, "default");
        assert!(!interface.readonly_mode_was_enabled);
    }

    #[test]
    fn test_enable_readonly_mode() {
        let mut logger = Logger::new(None, None);
        let mut interface = SystemInterface::new(&mut logger);

        interface.enable_readonly_mode();
        assert_eq!(interface.logger.logs.len(), 1);
        assert!(interface.logger.logs[0].contains("Enabling readonly mode..."));
    }

    #[test]
    fn test_disable_readonly_mode() {
        let mut logger = Logger::new(None, None);
        let mut interface = SystemInterface::new(&mut logger);

        interface.disable_readonly_mode();
        assert_eq!(interface.logger.logs.len(), 1);
        assert!(interface.logger.logs[0].contains("Disabling readonly mode..."));
    }

    #[test]
    fn test_mark_restore_as_running() {
        let mut logger = Logger::new(None, None);
        let mut interface = SystemInterface::new(&mut logger);

        interface.mark_restore_as_running();
        assert_eq!(interface.logger.logs.len(), 1);
        assert!(interface.logger.logs[0].contains("Marking restore as running..."));
    }
}
