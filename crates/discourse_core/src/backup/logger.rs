use chrono::Local;

pub const LOGS_CHANNEL: &str = "/backup-restore/logs";

#[derive(Default)]
pub struct Logger {
    user_id: Option<i32>,
    client_id: Option<String>,
    pub logs: Vec<String>,
}

impl Logger {
    pub fn new(user_id: Option<i32>, client_id: Option<String>) -> Self {
        Self {
            user_id,
            client_id,
            logs: Vec::new(),
        }
    }

    pub fn log(&mut self, message: &str, ex: Option<&dyn std::error::Error>) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // In a real implementation, we would check if we're in test environment
        // and optionally not print. We will just print for now.
        println!("{}", message);

        self.publish_log(message, &timestamp);
        self.save_log(message, &timestamp);

        if let Some(error) = ex {
            let formatted_ex = format!("{}\n", error);
            println!("{}", formatted_ex);
            // In a real implementation, this would use a proper logging framework like log/tracing
            // log::error!("{}", formatted_ex);
        }
    }

    fn publish_log(&self, _message: &str, _timestamp: &str) {
        if self.user_id.is_some() && self.client_id.is_some() {
            // In a real implementation, this would publish to a message bus
            // MessageBus::publish(LOGS_CHANNEL, data, user_ids, client_ids)
        }
    }

    fn save_log(&mut self, message: &str, timestamp: &str) {
        self.logs.push(format!("[{}] {}", timestamp, message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_initialization() {
        let logger = Logger::new(Some(1), Some("client1".to_string()));
        assert_eq!(logger.user_id, Some(1));
        assert_eq!(logger.client_id, Some("client1".to_string()));
        assert!(logger.logs.is_empty());
    }

    #[test]
    fn test_logger_log() {
        let mut logger = Logger::new(None, None);
        logger.log("Test message", None);

        assert_eq!(logger.logs.len(), 1);
        assert!(logger.logs[0].contains("Test message"));
        assert!(logger.logs[0].starts_with('['));
    }
}
