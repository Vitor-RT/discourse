use std::path::Path;
use std::fs;
use std::cell::RefCell;
use chrono::Local;
use url::Url;
use std::process::Command;
use crate::backup::logger::Logger;

pub struct BackupFileHandler<'a> {
    logger: &'a RefCell<Logger>,
    url: Option<String>,
    filename: String,
    current_db: String,
    root_tmp_directory: String,
    is_archive: bool,
    store_location: Option<String>,
    tmp_directory: Option<String>,
    archive_path: Option<String>,
    db_dump_path: Option<String>,
}

impl<'a> BackupFileHandler<'a> {
    pub const OLD_DUMP_FILENAME: &'static str = "dump.sql";
    pub const DUMP_FILE: &'static str = "dump.sql";

    pub fn new(
        logger: &'a RefCell<Logger>,
        filename: String,
        current_db: String,
        root_tmp_directory: String,
        location: Option<String>,
    ) -> Self {
        let (url, actual_filename) = if filename.starts_with("http://") || filename.starts_with("https://") {
            let parsed_url = Url::parse(&filename).unwrap();
            let name = parsed_url.path_segments().unwrap().last().unwrap().to_string();
            (Some(filename), name)
        } else {
            (None, filename)
        };

        let is_archive = !actual_filename.ends_with(".sql.gz");

        Self {
            logger,
            url,
            filename: actual_filename,
            current_db,
            root_tmp_directory,
            is_archive,
            store_location: location,
            tmp_directory: None,
            archive_path: None,
            db_dump_path: None,
        }
    }

    fn log(&self, message: &str) {
        self.logger.borrow_mut().log(message, None);
    }

    pub fn decompress(&mut self) -> Result<(String, String, String), Box<dyn std::error::Error>> {
        self.create_tmp_directory()?;

        let tmp_dir = self.tmp_directory.as_ref().unwrap().clone();
        let archive_path = Path::new(&tmp_dir).join(&self.filename);
        self.archive_path = Some(archive_path.to_string_lossy().to_string());

        if self.url.is_some() {
            self.download_archive_to_tmp_directory()?;
        } else {
            self.copy_archive_to_tmp_directory()?;
        }

        self.decompress_archive()?;
        self.extract_db_dump()?;

        Ok((
            self.filename.clone(),
            self.tmp_directory.clone().unwrap(),
            self.db_dump_path.clone().unwrap(),
        ))
    }

    pub fn clean_up(&self) {
        if let Some(ref tmp_dir) = self.tmp_directory {
            self.log(&format!("Removing tmp '{}' directory...", tmp_dir));
            if let Err(_e) = fs::remove_dir_all(tmp_dir) {
                self.log(&format!("Something went wrong while removing the following tmp directory: {}", tmp_dir));
            }
        }
    }

    fn create_tmp_directory(&mut self) -> Result<(), std::io::Error> {
        let timestamp = Local::now().format("%Y-%m-%d-%H%M%S").to_string();
        let tmp_dir = Path::new(&self.root_tmp_directory)
            .join("tmp")
            .join("restores")
            .join(&self.current_db)
            .join(timestamp);

        self.ensure_directory_exists(&tmp_dir)?;
        self.tmp_directory = Some(tmp_dir.to_string_lossy().to_string());
        Ok(())
    }

    fn ensure_directory_exists(&self, directory: &Path) -> Result<(), std::io::Error> {
        self.log(&format!("Making sure {} exists...", directory.display()));
        fs::create_dir_all(directory)
    }

    fn download_archive_to_tmp_directory(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.log("Downloading archive from URL to tmp directory...");
        // Placeholder for FileHelper.download equivalent
        Ok(())
    }

    fn copy_archive_to_tmp_directory(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Placeholder for BackupStore integration
        let is_remote = false;
        if is_remote {
            self.log("Downloading archive to tmp directory...");
        } else {
            self.log("Copying archive to tmp directory...");
        }
        Ok(())
    }

    fn tar_implementation(&self) -> String {
        let output = Command::new("tar").arg("--version").output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("GNU tar") {
            "gnu".to_string()
        } else if stdout.contains("bsdtar") {
            "bsd".to_string()
        } else {
            panic!("Unknown tar implementation: {}", stdout);
        }
    }

    fn decompress_archive(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_archive {
            return Ok(());
        }

        let tar_impl = self.tar_implementation();
        let mut path_transformation = Vec::new();
        if tar_impl == "gnu" {
            path_transformation.push("--transform");
            path_transformation.push("s|var/www/discourse/public/uploads/|uploads/|");
        } else if tar_impl == "bsd" {
            path_transformation.push("-s");
            path_transformation.push("|var/www/discourse/public/uploads/|uploads/|");
        }

        self.log("Unzipping archive, this may take a while...");
        let mut cmd = Command::new("tar");
        cmd.arg("--extract")
           .arg("--gzip")
           .arg("--file")
           .arg(self.archive_path.as_ref().unwrap())
           .arg("--directory")
           .arg(self.tmp_directory.as_ref().unwrap());

        for arg in path_transformation {
            cmd.arg(arg);
        }

        let status = cmd.status()?;
        if !status.success() {
            return Err("Failed to decompress archive.".into());
        }

        Ok(())
    }

    fn extract_db_dump(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let tmp_dir = self.tmp_directory.as_ref().unwrap();
        let db_dump_path = if self.is_archive {
            let old_dump_path = Path::new(tmp_dir).join(Self::OLD_DUMP_FILENAME);
            if old_dump_path.exists() {
                old_dump_path
            } else {
                Path::new(tmp_dir).join(Self::DUMP_FILE)
            }
        } else {
            Path::new(tmp_dir).join(&self.filename)
        };

        let mut db_dump_path_buf = db_dump_path.to_path_buf();

        if db_dump_path_buf.extension().and_then(|s| s.to_str()) == Some("gz") {
            self.log("Extracting dump file...");
            // Placeholder for gzip decompression
            db_dump_path_buf.set_extension("");
        }

        self.db_dump_path = Some(db_dump_path_buf.to_string_lossy().to_string());
        Ok(())
    }
}
