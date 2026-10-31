use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use tar::Archive;

use crate::compression::strategy::{
    calculate_available_size, is_safe_path_for_extraction, ExtractionError, Strategy,
};

pub struct Tar;

impl Strategy for Tar {
    fn extension(&self) -> &str {
        ".tar"
    }

    fn compress(&self, path: &Path, target_name: &str) -> Result<String, ExtractionError> {
        let tar_filename = format!("{}.tar", target_name);

        let output = Command::new("tar")
            .current_dir(path)
            .arg("--create")
            .arg("--file")
            .arg(&tar_filename)
            .arg(target_name)
            .output()
            .map_err(|e| ExtractionError::Other(format!("Failed to execute tar: {}", e)))?;

        if !output.status.success() {
            return Err(ExtractionError::Other("Failed to tar file.".to_string()));
        }

        let result_path = path.join(&tar_filename);
        Ok(result_path.to_string_lossy().to_string())
    }

    fn decompress(
        &self,
        dest_path: &Path,
        compressed_file_path: &Path,
        max_size: usize,
    ) -> Result<PathBuf, ExtractionError> {
        let file = File::open(compressed_file_path).map_err(ExtractionError::Io)?;
        let mut archive = Archive::new(file);

        let mut available_size = calculate_available_size(max_size);

        for entry_res in archive.entries().map_err(ExtractionError::Io)? {
            let mut entry = entry_res.map_err(ExtractionError::Io)?;

            let entry_path = entry.path().map_err(ExtractionError::Io)?;
            let dest_entry_path = dest_path.join(entry_path);

            if !is_safe_path_for_extraction(&dest_entry_path, dest_path) {
                continue;
            }

            let entry_size = entry.header().size().unwrap_or(0) as usize;
            if entry_size > available_size {
                return Err(ExtractionError::ExtractFailed);
            }
            available_size -= entry_size;

            entry.unpack(&dest_entry_path).map_err(ExtractionError::Io)?;
        }

        Ok(dest_path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_tar_compression_decompression() {
        let dir = tempdir().unwrap();
        let target_dir = dir.path().join("test_target");
        fs::create_dir(&target_dir).unwrap();

        let file_path = target_dir.join("test.txt");
        fs::write(&file_path, b"hello tar").unwrap();

        let tar_strat = Tar;
        let compressed_path_str = tar_strat.compress(dir.path(), "test_target").unwrap();
        let compressed_path = PathBuf::from(compressed_path_str);

        assert!(compressed_path.exists());

        let dest_dir = tempdir().unwrap();
        let decompressed_path = tar_strat
            .decompress(dest_dir.path(), &compressed_path, 10)
            .unwrap();

        assert!(decompressed_path.exists());
        let decompressed_file_path = decompressed_path.join("test_target/test.txt");
        assert!(decompressed_file_path.exists());
        let content = fs::read_to_string(decompressed_file_path).unwrap();
        assert_eq!(content, "hello tar");
    }
}
