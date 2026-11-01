use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use regex::Regex;
use std::cell::RefCell;
use crate::backup::logger::Logger;

#[derive(Error, Debug, PartialEq)]
pub enum MetaDataError {
    #[error("Version is not in a valid format.")]
    InvalidVersionFormat,
    #[error("Version information is missing or invalid in the filename.")]
    MissingOrInvalidVersionInFilename,
    #[error("Failed to load metadata file.")]
    LoadFailed,
    #[error("Failed to parse metadata file.")]
    ParseFailed,
    #[error("IO error: {0}")]
    IoError(String),
    #[error("JSON error: {0}")]
    JsonError(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum MigrationRequiredError {
    #[error("This backup was created with Discourse {backup_version}, but this site is running Discourse {site_version}. Upgrade this site before restoring the backup")]
    DiscourseVersion {
        backup_version: String,
        site_version: String,
    },
    #[error("This backup uses schema version {backup_schema}, but this site is on schema version {site_schema}. Upgrade this site before restoring the backup")]
    SchemaVersion {
        backup_schema: usize,
        site_schema: usize,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct MetaData {
    pub version: usize,
    pub discourse_version: Option<String>,
}

pub struct MetaDataHandler<'a> {
    logger: &'a RefCell<Logger>,
    filename: String,
    tmp_directory: String,
    current_database_version: usize,
    current_discourse_version: String,
    version_prefix: String,
}

// Custom parser to split version strings accurately
fn parse_version(version: &str) -> Vec<u32> {
    version.split(|c: char| !c.is_numeric())
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
}

fn version_greater_than(v1: &str, v2: &str) -> bool {
    let p1 = parse_version(v1);
    let p2 = parse_version(v2);

    for (a, b) in p1.iter().zip(p2.iter()) {
        if a > b {
            return true;
        } else if a < b {
            return false;
        }
    }

    p1.len() > p2.len()
}

impl<'a> MetaDataHandler<'a> {
    pub fn new(
        logger: &'a RefCell<Logger>,
        filename: String,
        tmp_directory: String,
        current_database_version: usize,
        current_discourse_version: String,
        version_prefix: String,
    ) -> Self {
        Self {
            logger,
            filename,
            tmp_directory,
            current_database_version,
            current_discourse_version,
            version_prefix,
        }
    }

    fn log(&self, message: &str) {
        self.logger.borrow_mut().log(message, None);
    }

    pub fn validate(&self) -> Result<MetaData, Box<dyn std::error::Error>> {
        let metadata = self.extract_metadata()?;

        self.log("Validating metadata...");
        self.log(&format!("  Current database version: {}", self.current_database_version));
        self.log(&format!("  Backup from database version: {}", metadata.version));
        self.log(&format!("  Current Discourse version: {}", self.current_discourse_version));

        let backup_discourse_version = metadata.discourse_version.as_deref().unwrap_or("(unknown)");
        self.log(&format!("  Backup from Discourse version: {}", backup_discourse_version));

        if let Some(ref dv) = metadata.discourse_version {
            if version_greater_than(dv, &self.current_discourse_version) {
                return Err(Box::new(MigrationRequiredError::DiscourseVersion {
                    backup_version: dv.clone(),
                    site_version: self.current_discourse_version.clone(),
                }));
            }
        }

        if metadata.version > self.current_database_version {
            return Err(Box::new(MigrationRequiredError::SchemaVersion {
                backup_schema: metadata.version,
                site_schema: self.current_database_version,
            }));
        }

        Ok(metadata)
    }

    fn extract_metadata(&self) -> Result<MetaData, Box<dyn std::error::Error>> {
        let metadata_path = if !self.tmp_directory.is_empty() {
            Some(Path::new(&self.tmp_directory).join("meta.json"))
        } else {
            None
        };

        if let Some(path) = metadata_path {
            if path.exists() {
                return Ok(self.load_metadata_file(&path)?);
            }
        }

        Ok(self.extract_filename_metadata()?)
    }

    fn extract_filename_metadata(&self) -> Result<MetaData, MetaDataError> {
        let version_regexp = r"\d{4}-\d{1,2}-\d+(?:-latest(?:-\d+)?)?";
        let pattern = format!(
            r"-{}-(?:(?P<discourse_version>{})-)?(?P<database_version>\d{{14}})",
            regex::escape(&self.version_prefix),
            version_regexp
        );
        let re = Regex::new(&pattern).unwrap();

        if let Some(caps) = re.captures(&self.filename) {
            let database_version_str = caps.name("database_version").unwrap().as_str();
            let version = database_version_str.parse::<usize>().unwrap();

            let discourse_version = caps.name("discourse_version").map(|m| {
                m.as_str().replace("-", ".").replace(".latest", "-latest")
            });

            Ok(MetaData {
                version,
                discourse_version,
            })
        } else {
            Err(MetaDataError::MissingOrInvalidVersionInFilename)
        }
    }

    fn load_metadata_file(&self, path: &Path) -> Result<MetaData, MetaDataError> {
        let content = fs::read_to_string(path).map_err(|e| MetaDataError::IoError(e.to_string()))?;
        if content.trim().is_empty() {
            return Err(MetaDataError::LoadFailed);
        }
        let metadata: MetaData = serde_json::from_str(&content).map_err(|e| MetaDataError::JsonError(e.to_string()))?;
        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_greater_than() {
        assert!(version_greater_than("3.10.0", "3.2.0"));
        assert!(!version_greater_than("3.2.0", "3.10.0"));
        assert!(version_greater_than("4.0.0", "3.9.9"));
        assert!(!version_greater_than("1.0.0", "1.0.0"));
    }

    #[test]
    fn test_extract_filename_metadata() {
        let logger = RefCell::new(Logger::new(None, None));
        let handler = MetaDataHandler::new(
            &logger,
            "backup-2023-10-25-010101-v-2023-10-25-12345678901234.tar.gz".to_string(),
            "".to_string(),
            20231025000000,
            "3.1.0".to_string(),
            "v".to_string(),
        );

        let metadata = handler.extract_filename_metadata().unwrap();
        assert_eq!(metadata.version, 12345678901234);
        assert_eq!(metadata.discourse_version, Some("2023.10.25".to_string()));
    }
}
