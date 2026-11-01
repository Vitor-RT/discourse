use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use super::strategy::{Strategy, CompressionError};

pub struct Gzip;

impl Strategy for Gzip {
    fn extension(&self) -> &'static str {
        ".gz"
    }

    fn compress(&self, path: &Path, target_name: &str) -> Result<String, CompressionError> {
        let source_path = path.join(target_name);
        let gzip_target = format!("{}.gz", source_path.to_string_lossy());

        let mut source_file = File::open(&source_path)?;
        let target_file = File::create(&gzip_target)?;

        let mut encoder = GzEncoder::new(target_file, Compression::new(5));

        let mut buffer = [0; 8192];
        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            encoder.write_all(&buffer[..bytes_read])?;
        }
        encoder.finish()?;

        Ok(gzip_target)
    }

    fn decompress(
        &self,
        dest_path: &Path,
        compressed_file_path: &Path,
        max_size: u64,
    ) -> Result<String, CompressionError> {
        std::fs::create_dir_all(dest_path)?;

        let file_name = compressed_file_path
            .file_name()
            .ok_or_else(|| CompressionError::Other("Invalid file name".to_string()))?
            .to_string_lossy()
            .to_string();

        let basename = if file_name.ends_with(".gz") {
            &file_name[..file_name.len() - 3]
        } else {
            &file_name
        };

        let entry_path = dest_path.join(basename);

        if entry_path.exists() {
            return Err(CompressionError::DestinationFileExistsError(format!(
                "Destination '{}' already exists",
                entry_path.display()
            )));
        }

        let compressed_file = File::open(compressed_file_path)?;
        let mut decoder = GzDecoder::new(compressed_file);

        let mut target_file = File::create(&entry_path)?;

        let mut remaining_size = max_size;
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = decoder.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            if bytes_read as u64 > remaining_size {
                return Err(CompressionError::ExtractFailed);
            }
            remaining_size -= bytes_read as u64;

            target_file.write_all(&buffer[..bytes_read])?;
        }

        Ok(entry_path.to_string_lossy().to_string())
    }
}
