use crate::backup::backup_file_handler::BackupFileHandler;
use crate::backup::logger::Logger;
use crate::backup::meta_data_handler::MetaDataHandler;
use std::cell::RefCell;

pub struct UploadsRestorer<'a> {
    _logger: &'a RefCell<Logger>,
}

pub struct DatabaseRestorer<'a> {
    _logger: &'a RefCell<Logger>,
    _current_db: String,
}

pub struct Factory {
    pub user_id: Option<i32>,
    pub client_id: Option<String>,
    logger: RefCell<Logger>,
}

impl Factory {
    pub fn new(user_id: Option<i32>, client_id: Option<String>) -> Self {
        let logger = Logger::new(user_id, client_id.clone());
        Self {
            user_id,
            client_id,
            logger: RefCell::new(logger),
        }
    }

    pub fn logger(&self) -> std::cell::RefMut<'_, Logger> {
        self.logger.borrow_mut()
    }

    pub fn meta_data_handler<'a>(
        &'a self,
        filename: String,
        tmp_directory: String,
        current_database_version: usize,
        current_discourse_version: String,
        version_prefix: String,
    ) -> MetaDataHandler<'a> {
        MetaDataHandler::new(
            &self.logger,
            filename,
            tmp_directory,
            current_database_version,
            current_discourse_version,
            version_prefix,
        )
    }

    pub fn backup_file_handler<'a>(
        &'a self,
        filename: String,
        current_db: String,
        root_tmp_directory: String,
        location: Option<String>,
    ) -> BackupFileHandler<'a> {
        BackupFileHandler::new(
            &self.logger,
            filename,
            current_db,
            root_tmp_directory,
            location,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_initialization() {
        let factory = Factory::new(Some(42), Some("client_xyz".to_string()));
        assert_eq!(factory.user_id, Some(42));
        assert_eq!(factory.client_id, Some("client_xyz".to_string()));
    }

    #[test]
    fn test_factory_logger() {
        let factory = Factory::new(None, None);
        let mut logger = factory.logger();
        logger.log("test", None);
        assert_eq!(logger.logs.len(), 1);
    }
}
