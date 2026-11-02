use std::fs;
use std::io;
use std::path::PathBuf;
use walkdir::WalkDir;

pub trait Importer {
    fn import(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    fn read_file(&self, relative: &str) -> Option<String> {
        let fullpath = self.real_path(relative)?;
        fs::read_to_string(fullpath).ok()
    }

    fn file_size(&self, relative: &str) -> i64 {
        if let Some(fullpath) = self.real_path(relative) {
            if let Ok(metadata) = fs::metadata(fullpath) {
                return metadata.len() as i64;
            }
        }
        -1
    }

    fn all_files(&self) -> Vec<String> {
        let mut files = Vec::new();
        let folder = self.temp_folder();

        if !folder.exists() {
            return files;
        }

        for entry in WalkDir::new(&folder).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Ok(rel_path) = entry.path().strip_prefix(&folder) {
                    files.push(rel_path.to_string_lossy().to_string());
                }
            }
        }
        files
    }

    fn cleanup(&self) -> io::Result<()> {
        let folder = self.temp_folder();
        if folder.exists() {
            fs::remove_dir_all(folder)?;
        }
        Ok(())
    }

    fn real_path(&self, relative: &str) -> Option<PathBuf> {
        let folder = self.temp_folder();
        let fullpath = folder.join(relative);

        if !fullpath.exists() {
            return None;
        }

        let canonical = match fullpath.canonicalize() {
            Ok(p) => p,
            Err(_) => return None,
        };

        let folder_canonical = match folder.canonicalize() {
            Ok(p) => p,
            Err(_) => return None,
        };

        if canonical.starts_with(&folder_canonical) {
            Some(canonical)
        } else {
            None
        }
    }

    fn temp_folder(&self) -> PathBuf;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct TestImporter {
        temp_dir: TempDir,
    }

    impl Importer for TestImporter {
        fn import(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn temp_folder(&self) -> PathBuf {
            self.temp_dir.path().to_path_buf()
        }
    }

    #[test]
    fn test_real_path_security() {
        let temp_dir = tempfile::tempdir().unwrap();
        let importer = TestImporter { temp_dir };
        let folder = importer.temp_folder();

        let safe_file = folder.join("safe.txt");
        fs::write(&safe_file, "safe content").unwrap();

        assert!(importer.real_path("safe.txt").is_some());

        // Attempt path traversal
        assert!(importer.real_path("../some_file.txt").is_none());
    }

    #[test]
    fn test_all_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let importer = TestImporter { temp_dir };
        let folder = importer.temp_folder();

        fs::write(folder.join("file1.txt"), "content").unwrap();
        fs::create_dir(folder.join("subdir")).unwrap();
        fs::write(folder.join("subdir").join("file2.txt"), "content").unwrap();

        let mut files = importer.all_files();
        files.sort();

        assert_eq!(files.len(), 2);

        assert!(files.contains(&"file1.txt".to_string()));
        #[cfg(unix)]
        assert!(files.contains(&"subdir/file2.txt".to_string()));
        #[cfg(windows)]
        assert!(files.contains(&"subdir\\file2.txt".to_string()));
    }
}
