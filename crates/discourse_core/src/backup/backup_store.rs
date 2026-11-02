use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BackupStoreError {
    #[error("Backup file exists")]
    BackupFileExists,
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Not implemented")]
    NotImplemented,
}

#[derive(Debug, Clone)]
pub struct BackupFile {
    pub filename: String,
    pub last_modified: DateTime<Utc>,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct StorageStats {
    pub used_bytes: usize,
    pub free_bytes: usize,
    pub count: usize,
    pub last_backup_taken_at: Option<DateTime<Utc>>,
}

pub trait BackupStore {
    fn unsorted_files(&self) -> Result<Vec<BackupFile>, BackupStoreError>;

    fn files(&self) -> Result<Vec<BackupFile>, BackupStoreError> {
        let mut files = self.unsorted_files()?;
        files.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        Ok(files)
    }

    fn latest_file(&self) -> Result<Option<BackupFile>, BackupStoreError> {
        Ok(self.files()?.into_iter().next())
    }

    fn remote(&self) -> bool {
        false
    }

    fn file(
        &self,
        filename: &str,
        include_download_source: bool,
    ) -> Result<Option<BackupFile>, BackupStoreError>;

    fn delete_file(&self, filename: &str) -> Result<(), BackupStoreError>;

    fn download_file(
        &self,
        filename: &str,
        destination: &str,
        failure_message: Option<&str>,
    ) -> Result<(), BackupStoreError>;

    fn upload_file(
        &self,
        filename: &str,
        source_path: &str,
        content_type: &str,
    ) -> Result<(), BackupStoreError>;

    fn cleanup_allowed(&self) -> bool {
        true
    }

    fn used_bytes(&self) -> Result<usize, BackupStoreError> {
        Ok(self.files()?.iter().map(|f| f.size).sum())
    }

    fn free_bytes(&self) -> Result<usize, BackupStoreError>;

    fn stats(&self) -> Result<StorageStats, BackupStoreError> {
        let files = self.files()?;
        let used_bytes = files.iter().map(|f| f.size).sum();
        let free_bytes = self.free_bytes()?;
        let count = files.len();
        let last_backup_taken_at = files.first().map(|f| f.last_modified.clone());

        Ok(StorageStats {
            used_bytes,
            free_bytes,
            count,
            last_backup_taken_at,
        })
    }

    fn delete_old(&self, maximum_backups: usize) -> Result<(), BackupStoreError> {
        if !self.cleanup_allowed() {
            return Ok(());
        }
        let files = self.files()?;
        if files.len() <= maximum_backups {
            return Ok(());
        }

        for file in files.iter().skip(maximum_backups) {
            self.delete_file(&file.filename)?;
        }
        Ok(())
    }

    fn delete_prior_to_n_days(&self, window_days: Option<usize>) -> Result<(), BackupStoreError> {
        let window = match window_days {
            Some(w) if w > 0 => w,
            _ => return Ok(()),
        };
        if !self.cleanup_allowed() {
            return Ok(());
        }

        let files = self.files()?;
        let cutoff = Utc::now() - chrono::Duration::days(window as i64);

        for file in files.iter() {
            if file.last_modified < cutoff {
                self.delete_file(&file.filename)?;
            }
        }
        Ok(())
    }
}
