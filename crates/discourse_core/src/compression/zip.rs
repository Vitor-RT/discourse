use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::compression::safe_zip_reader::SafeZipReader;
use crate::compression::strategy::{
    calculate_available_size, is_safe_path_for_extraction, ExtractionError, Strategy,
    DEFAULT_CHUNK_SIZE,
};

pub const MAX_ZIP_ENTRIES: usize = 10_000;

pub struct Zip;

impl Strategy for Zip {
    fn extension(&self) -> &str {
        ".zip"
    }

    fn compress(&self, path: &Path, target_name: &str) -> Result<String, ExtractionError> {
        let target_path = path.join(target_name);
        let zip_filename = format!("{}.zip", target_path.to_string_lossy());
        let zip_file = File::create(&zip_filename).map_err(ExtractionError::Io)?;
        let mut zip = ZipWriter::new(zip_file);

        let options: FileOptions<()> = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        if target_path.is_dir() {
            let mut stack = vec![target_path.clone()];
            while let Some(current_dir) = stack.pop() {
                for entry_res in fs::read_dir(&current_dir).map_err(ExtractionError::Io)? {
                    let entry = entry_res.map_err(ExtractionError::Io)?;
                    let file_path = entry.path();
                    let relative_path = file_path
                        .strip_prefix(&target_path)
                        .unwrap_or(&file_path)
                        .to_string_lossy()
                        .into_owned();

                    if file_path.is_dir() {
                        zip.add_directory(&relative_path, options.clone())
                            .map_err(|e| ExtractionError::Other(e.to_string()))?;
                        stack.push(file_path);
                    } else {
                        zip.start_file(&relative_path, options.clone())
                            .map_err(|e| ExtractionError::Other(e.to_string()))?;
                        let mut f = File::open(&file_path).map_err(ExtractionError::Io)?;
                        let mut buffer = Vec::new();
                        f.read_to_end(&mut buffer).map_err(ExtractionError::Io)?;
                        zip.write_all(&buffer).map_err(ExtractionError::Io)?;
                    }
                }
            }
        } else {
            zip.start_file(target_name, options)
                .map_err(|e| ExtractionError::Other(e.to_string()))?;
            let mut f = File::open(&target_path).map_err(ExtractionError::Io)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).map_err(ExtractionError::Io)?;
            zip.write_all(&buffer).map_err(ExtractionError::Io)?;
        }

        zip.finish().map_err(|e| ExtractionError::Other(e.to_string()))?;

        Ok(zip_filename)
    }

    fn decompress(
        &self,
        dest_path: &Path,
        compressed_file_path: &Path,
        max_size: usize,
    ) -> Result<PathBuf, ExtractionError> {
        let file = File::open(compressed_file_path).map_err(ExtractionError::Io)?;
        let archive = ZipArchive::new(file).map_err(|e| ExtractionError::Other(e.to_string()))?;

        let safe_reader = SafeZipReader::new(archive, Some(MAX_ZIP_ENTRIES), None, None)
            .map_err(|e| ExtractionError::Other(format!("{:?}", e)))?;

        let mut zip_archive = safe_reader.archive;

        let mut available_size = calculate_available_size(max_size);

        for i in 0..zip_archive.len() {
            let mut file = zip_archive
                .by_index(i)
                .map_err(|e| ExtractionError::Other(e.to_string()))?;

            let outpath = match file.enclosed_name() {
                Some(path) => dest_path.join(path),
                None => continue,
            };

            if !is_safe_path_for_extraction(&outpath, dest_path) {
                continue;
            }

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath).map_err(ExtractionError::Io)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p).map_err(ExtractionError::Io)?;
                    }
                }

                if outpath.exists() {
                    return Err(ExtractionError::DestinationFileExistsError(
                        outpath.to_string_lossy().to_string(),
                    ));
                }

                let mut outfile = File::create(&outpath).map_err(ExtractionError::Io)?;
                let mut buffer = vec![0; DEFAULT_CHUNK_SIZE];

                loop {
                    let bytes_read = file.read(&mut buffer).map_err(ExtractionError::Io)?;
                    if bytes_read == 0 {
                        break;
                    }

                    if bytes_read > available_size {
                        return Err(ExtractionError::ExtractFailed);
                    }
                    available_size -= bytes_read;

                    outfile
                        .write_all(&buffer[..bytes_read])
                        .map_err(ExtractionError::Io)?;
                }
            }

            // Get and Set permissions if necessary (omitted for brevity, typically supported via file.unix_mode())
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).ok();
                }
            }
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
    fn test_zip_compression_decompression() {
        let dir = tempdir().unwrap();
        let target_dir = dir.path().join("test_target");
        fs::create_dir(&target_dir).unwrap();

        let file_path = target_dir.join("test.txt");
        fs::write(&file_path, b"hello zip").unwrap();

        let zip_strat = Zip;
        let compressed_path_str = zip_strat.compress(dir.path(), "test_target").unwrap();
        let compressed_path = PathBuf::from(compressed_path_str);

        assert!(compressed_path.exists());

        let dest_dir = tempdir().unwrap();
        let decompressed_path = zip_strat
            .decompress(dest_dir.path(), &compressed_path, 10)
            .unwrap();

        assert!(decompressed_path.exists());
        let decompressed_file_path = decompressed_path.join("test.txt");
        assert!(decompressed_file_path.exists());
        let content = fs::read_to_string(decompressed_file_path).unwrap();
        assert_eq!(content, "hello zip");
    }
}
