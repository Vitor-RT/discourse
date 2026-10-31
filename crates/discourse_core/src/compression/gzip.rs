use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::read::GzDecoder;

use crate::compression::strategy::{
    calculate_available_size, is_safe_path_for_extraction, ExtractionError, Strategy,
    DEFAULT_CHUNK_SIZE,
};

pub struct Gzip;

impl Strategy for Gzip {
    fn extension(&self) -> &str {
        ".gz"
    }

    fn compress(&self, path: &Path, target_name: &str) -> Result<String, ExtractionError> {
        let gzip_target = path.join(target_name);

        let output = Command::new("gzip")
            .arg("-5")
            .arg(&gzip_target)
            .output()
            .map_err(|e| ExtractionError::Other(format!("Failed to execute gzip: {}", e)))?;

        if !output.status.success() {
            return Err(ExtractionError::Other("Failed to gzip file.".to_string()));
        }

        let mut gzip_target_str = gzip_target.to_string_lossy().to_string();
        gzip_target_str.push_str(".gz");
        Ok(gzip_target_str)
    }

    fn decompress(
        &self,
        dest_path: &Path,
        compressed_file_path: &Path,
        max_size: usize,
    ) -> Result<PathBuf, ExtractionError> {
        let file = File::open(compressed_file_path).map_err(ExtractionError::Io)?;
        let mut decoder = GzDecoder::new(file);

        let mut available_size = calculate_available_size(max_size);

        let file_name = compressed_file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let file_name_without_ext = if file_name.ends_with(self.extension()) {
            &file_name[..file_name.len() - self.extension().len()]
        } else {
            file_name
        };

        let dest_file_path = dest_path.join(file_name_without_ext);

        if !is_safe_path_for_extraction(&dest_file_path, dest_path) {
            return Err(ExtractionError::Other(
                "Unsafe extraction path".to_string(),
            ));
        }

        if dest_file_path.exists() {
            return Err(ExtractionError::DestinationFileExistsError(
                dest_file_path.to_string_lossy().to_string(),
            ));
        }

        let mut out_file = File::create(&dest_file_path).map_err(ExtractionError::Io)?;
        let mut buffer = vec![0; DEFAULT_CHUNK_SIZE];

        loop {
            let bytes_read = decoder.read(&mut buffer).map_err(ExtractionError::Io)?;
            if bytes_read == 0 {
                break;
            }

            if bytes_read > available_size {
                return Err(ExtractionError::ExtractFailed);
            }
            available_size -= bytes_read;

            out_file
                .write_all(&buffer[..bytes_read])
                .map_err(ExtractionError::Io)?;
        }

        Ok(dest_file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_gzip_compression_decompression() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, b"hello world").unwrap();

        let gzip = Gzip;
        let compressed_path_str = gzip.compress(dir.path(), "test.txt").unwrap();
        let compressed_path = PathBuf::from(compressed_path_str);

        assert!(compressed_path.exists());

        let dest_dir = tempdir().unwrap();
        let decompressed_path = gzip
            .decompress(dest_dir.path(), &compressed_path, 10)
            .unwrap();

        assert!(decompressed_path.exists());
        let content = fs::read_to_string(decompressed_path).unwrap();
        assert_eq!(content, "hello world");
    }
}
