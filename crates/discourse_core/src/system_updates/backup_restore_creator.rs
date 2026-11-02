use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

//
// Traits for dependencies
//

pub trait FileStore: Send + Sync {
    fn remote(&self) -> bool;
    fn get_path_for_upload(&self, upload_data: &UploadData) -> String;
    fn download_file(&self, upload_data: &UploadData, filename: &str) -> Result<(), String>;
    fn upload_file(&self, filename: &str, archive_path: &str, content_type: &str) -> Result<(), String>;
    fn delete_old(&self) -> Result<(), String>;
    fn reset_cache(&self) -> Result<(), String>;
}

pub trait DbClient: Send + Sync {
    fn query_uploads(&self) -> Vec<UploadData>;
    fn mark_as_not_running(&self) -> Result<(), String>;
}

pub trait Logger: Send + Sync {
    fn log(&self, message: &str);
    fn publish_log(&self, message: &str, timestamp: &str);
    fn save_log(&self, message: &str, timestamp: &str);
}

pub trait MessageBus: Send + Sync {
    fn publish(&self, channel: &str, data: HashMap<String, String>);
}

pub trait ArchiveWriter: Send + Sync {
    fn add_file(&self, src_path: &str, dest_path: &str, mode: u32) -> Result<(), String>;
    fn add_hardlink(&self, dest_path: &str, src_path: &str) -> Result<(), String>;
}

//
// Models
//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadData {
    pub id: i32,
    pub url: String,
    pub sha1: String,
    pub extension: String,
    pub original_filename: String,
}

#[derive(Debug, Default)]
pub struct DownloadStats {
    pub downloaded: usize,
    pub hardlinked: usize,
    pub copied: usize,
}

//
// Creator core logic
//

pub struct CreatorContext {
    pub file_store: Arc<dyn FileStore>,
    pub db: Arc<dyn DbClient>,
    pub logger: Arc<dyn Logger>,
    pub message_bus: Arc<dyn MessageBus>,
    pub archive: Arc<dyn ArchiveWriter>,

    pub upload_directory: String,
    pub tmp_directory: String,
    pub archive_directory: String,
    pub backup_filename: String,

    pub include_s3_uploads: bool,
    pub download_concurrency: usize,

    pub success: bool,
    pub user_id: i32,
    pub system_user_id: i32,
    pub publish_to_message_bus: bool,
    pub is_development: bool,
}

pub struct Creator {
    ctx: CreatorContext,
    download_stats: Arc<Mutex<DownloadStats>>,
    total_uploads: usize,
}

impl Creator {
    pub const MAX_LINKS: usize = 100; // Arbitrary for example

    pub fn new(ctx: CreatorContext) -> Self {
        Self {
            ctx,
            download_stats: Arc::new(Mutex::new(DownloadStats::default())),
            total_uploads: 0,
        }
    }

    pub fn add_remote_uploads_to_archive(&mut self) {
        if !self.ctx.include_s3_uploads {
            self.ctx.logger.log("Skipping uploads stored on S3.");
            return;
        }

        self.ctx.logger.log("Downloading uploads from S3. This may take a while...");

        let uploads = self.ctx.db.query_uploads();
        let uploads_by_sha1 = self.group_remote_uploads_by_sha1(uploads);

        self.total_uploads = uploads_by_sha1.values().map(|g| g.len()).sum();
        self.ctx.logger.log(&format!(
            "Found {} unique files from {} total uploads",
            uploads_by_sha1.len(),
            self.total_uploads
        ));

        let groups: Vec<Vec<UploadData>> = uploads_by_sha1.into_values().collect();
        self.download_upload_groups(groups);

        let stats = self.download_stats.lock().unwrap();
        self.ctx.logger.log(&format!(
            "Finished processing uploads: {} downloaded, {} hardlinked, {} copied",
            stats.downloaded, stats.hardlinked, stats.copied
        ));

        if stats.downloaded == 0 && stats.hardlinked == 0 && stats.copied == 0 {
            self.ctx.logger.log("No uploads found on S3. Skipping archiving of uploads stored on S3...");
        }
    }

    fn group_remote_uploads_by_sha1(&self, uploads: Vec<UploadData>) -> HashMap<String, Vec<UploadData>> {
        let mut by_sha1: HashMap<String, Vec<UploadData>> = HashMap::new();
        for u in uploads {
            // Ruby did COALESCE logic, we assume the db query handles returning the right sha1 in `sha1`
            let key = u.sha1.clone();
            by_sha1.entry(key).or_default().push(u);
        }
        by_sha1
    }

    fn download_upload_groups(&self, upload_groups: Vec<Vec<UploadData>>) {
        let concurrency = self.ctx.download_concurrency.max(1);
        self.ctx.logger.log(&format!("Downloading upload groups with {} worker(s)", concurrency));

        // Simplified concurrency logic for translation: loop synchronously or use simple Rayon/Threads
        // Since we are mocking/translating logic, we'll do a simple iteration that mimics the behavior
        // Real implementation would spawn threads and use crossbeam or mpsc channels.

        for group in upload_groups {
            self.process_upload_group(group);
        }
    }

    fn process_upload_group(&self, upload_group: Vec<UploadData>) {
        if upload_group.is_empty() { return; }
        let primary = &upload_group[0];
        let tmp_file = format!("{}/backup-upload-{}.tmp", self.ctx.tmp_directory, primary.id);

        if self.download_upload_to_file(primary, &tmp_file) {
            let primary_path = self.upload_path_in_archive(primary);

            // Assume the archive handles mutex internally or we are serialized here
            let _ = self.ctx.archive.add_file(&tmp_file, &primary_path, 0o644);

            let mut archived_paths = HashSet::new();
            archived_paths.insert(primary_path.clone());

            let mut link_count = 1;
            let mut current_primary_path = primary_path;

            for duplicate in upload_group.iter().skip(1) {
                let path = self.upload_path_in_archive(duplicate);
                if !archived_paths.insert(path.clone()) {
                    continue;
                }

                if link_count >= Self::MAX_LINKS {
                    let _ = self.ctx.archive.add_file(&tmp_file, &path, 0o644);
                    current_primary_path = path;
                    link_count = 1;
                    self.increment_and_log_progress("copied");
                } else {
                    let _ = self.ctx.archive.add_hardlink(&path, &current_primary_path);
                    link_count += 1;
                    self.increment_and_log_progress("hardlinked");
                }
            }
        }
    }

    fn upload_path_in_archive(&self, upload_data: &UploadData) -> String {
        format!("{}/{}", self.ctx.upload_directory, self.ctx.file_store.get_path_for_upload(upload_data))
    }

    fn download_upload_to_file(&self, upload_data: &UploadData, filename: &str) -> bool {
        match self.ctx.file_store.download_file(upload_data, filename) {
            Ok(_) => {
                self.increment_and_log_progress("downloaded");
                true
            }
            Err(e) => {
                self.ctx.logger.log(&format!("Failed to download file with upload ID {} from S3: {}", upload_data.id, e));
                false
            }
        }
    }

    fn increment_and_log_progress(&self, stat_type: &str) {
        let mut stats = self.download_stats.lock().unwrap();
        match stat_type {
            "downloaded" => stats.downloaded += 1,
            "hardlinked" => stats.hardlinked += 1,
            "copied" => stats.copied += 1,
            _ => {}
        }

        let processed = stats.downloaded + stats.hardlinked + stats.copied;
        if processed % 1000 == 0 {
            let percentage = if self.total_uploads > 0 {
                (processed * 100) / self.total_uploads
            } else {
                0
            };
            self.ctx.logger.log(&format!(
                "{}/{} ({}%) files processed ({} downloaded, {} hardlinked, {} copied). Still processing...",
                processed, self.total_uploads, percentage, stats.downloaded, stats.hardlinked, stats.copied
            ));
        }
    }

    pub fn upload_archive(&self) {
        if !self.ctx.file_store.remote() {
            return;
        }

        self.ctx.logger.log("Uploading archive...");
        let content_type = "application/gzip"; // Hardcoded default based on ruby
        let archive_path = format!("{}/{}", self.ctx.archive_directory, self.ctx.backup_filename);
        let _ = self.ctx.file_store.upload_file(&self.ctx.backup_filename, &archive_path, content_type);
    }

    pub fn delete_old(&self) {
        if self.ctx.is_development {
            return;
        }

        self.ctx.logger.log("Deleting old backups...");
        if let Err(e) = self.ctx.file_store.delete_old() {
            self.ctx.logger.log(&format!("Something went wrong while deleting old backups: {}", e));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockFileStore {
        pub remote: bool,
    }

    impl FileStore for MockFileStore {
        fn remote(&self) -> bool { self.remote }
        fn get_path_for_upload(&self, u: &UploadData) -> String { format!("{}_{}", u.id, u.original_filename) }
        fn download_file(&self, _u: &UploadData, _f: &str) -> Result<(), String> { Ok(()) }
        fn upload_file(&self, _f: &str, _a: &str, _c: &str) -> Result<(), String> { Ok(()) }
        fn delete_old(&self) -> Result<(), String> { Ok(()) }
        fn reset_cache(&self) -> Result<(), String> { Ok(()) }
    }

    #[derive(Default)]
    struct MockDbClient {
        pub uploads: Vec<UploadData>,
    }

    impl DbClient for MockDbClient {
        fn query_uploads(&self) -> Vec<UploadData> { self.uploads.clone() }
        fn mark_as_not_running(&self) -> Result<(), String> { Ok(()) }
    }

    #[derive(Default)]
    struct MockLogger {
        pub logs: Mutex<Vec<String>>,
    }

    impl Logger for MockLogger {
        fn log(&self, m: &str) { self.logs.lock().unwrap().push(m.to_string()); }
        fn publish_log(&self, _m: &str, _t: &str) {}
        fn save_log(&self, _m: &str, _t: &str) {}
    }

    #[derive(Default)]
    struct MockMessageBus;
    impl MessageBus for MockMessageBus {
        fn publish(&self, _c: &str, _d: HashMap<String, String>) {}
    }

    #[derive(Default)]
    struct MockArchiveWriter {
        pub files: Mutex<Vec<(String, String)>>,
        pub links: Mutex<Vec<(String, String)>>,
    }

    impl ArchiveWriter for MockArchiveWriter {
        fn add_file(&self, s: &str, d: &str, _m: u32) -> Result<(), String> {
            self.files.lock().unwrap().push((s.to_string(), d.to_string()));
            Ok(())
        }
        fn add_hardlink(&self, d: &str, s: &str) -> Result<(), String> {
            self.links.lock().unwrap().push((d.to_string(), s.to_string()));
            Ok(())
        }
    }

    #[test]
    fn test_add_remote_uploads_to_archive() {
        let mut db = MockDbClient::default();
        db.uploads = vec![
            UploadData { id: 1, url: "".into(), sha1: "abc".into(), extension: "jpg".into(), original_filename: "1.jpg".into() },
            UploadData { id: 2, url: "".into(), sha1: "abc".into(), extension: "jpg".into(), original_filename: "2.jpg".into() },
        ];

        let archive = Arc::new(MockArchiveWriter::default());

        let ctx = CreatorContext {
            file_store: Arc::new(MockFileStore::default()),
            db: Arc::new(db),
            logger: Arc::new(MockLogger::default()),
            message_bus: Arc::new(MockMessageBus::default()),
            archive: archive.clone(),
            upload_directory: "/uploads".into(),
            tmp_directory: "/tmp".into(),
            archive_directory: "/archive".into(),
            backup_filename: "backup.tar.gz".into(),
            include_s3_uploads: true,
            download_concurrency: 1,
            success: true,
            user_id: 1,
            system_user_id: -1,
            publish_to_message_bus: false,
            is_development: false,
        };

        let mut creator = Creator::new(ctx);
        creator.add_remote_uploads_to_archive();

        let stats = creator.download_stats.lock().unwrap();
        assert_eq!(stats.downloaded, 1);
        assert_eq!(stats.hardlinked, 1);
        assert_eq!(stats.copied, 0);

        let files = archive.files.lock().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].1, "/uploads/1_1.jpg");

        let links = archive.links.lock().unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, "/uploads/2_2.jpg");
    }
}