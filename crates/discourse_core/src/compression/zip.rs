use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::{ZipArchive, ZipWriter};
use zip::write::SimpleFileOptions;
use walkdir::WalkDir;
use super::strategy::{Strategy, CompressionError};
use super::safe_zip_reader::SafeZipReader;

pub struct Zip;

impl Strategy for Zip {
    fn extension(&self) -> &'static str {
        ".zip"
    }

    fn compress(&self, path: &Path, target_name: &str) -> Result<String, CompressionError> {
        let source_path = path.join(target_name);
        let zip_filename = format!("{}.zip", source_path.to_string_lossy());
        let target_path = Path::new(&zip_filename);
        
        let file = File::create(target_path)?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
            
        if source_path.is_dir() {
            for entry in WalkDir::new(&source_path) {
                let entry = entry.map_err(|e| CompressionError::Other(e.to_string()))?;
                let path = entry.path();
                let name = path.strip_prefix(&source_path).map_err(|e| CompressionError::Other(e.to_string()))?;
                
                if name.as_os_str().is_empty() {
                    continue;
                }
                
                // Ruby code uses forward slashes in zip archives
                let name_str = name.to_string_lossy().replace("\\", "/");
                
                if path.is_file() {
                    zip.start_file(name_str, options)?;
                    let mut f = File::open(path)?;
                    let mut buffer = Vec::new();
                    f.read_to_end(&mut buffer)?;
                    zip.write_all(&buffer)?;
                } else if path.is_dir() {
                    zip.add_directory(name_str, options)?;
                }
            }
        } else {
            zip.start_file(target_name, options)?;
            let mut f = File::open(&source_path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
        
        zip.finish()?;
        
        Ok(zip_filename)
    }

    fn decompress(&self, dest_path: &Path, compressed_file_path: &Path, max_size: u64) -> Result<String, CompressionError> {
        let file = File::open(compressed_file_path)?;
        let mut archive = SafeZipReader::new(file, Some(10_000), Some(max_size), None)?;
        
        std::fs::create_dir_all(dest_path)?;
        let canonical_dest = dest_path.canonicalize()?;
        
        let chunk_size = 1024 * 1024 * 2; // 2MiB
        let mut remaining_size = max_size;
        
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let entry_name = entry.name().to_string();
            
            // Skip symlinks (not implemented here safely, matching ruby code intent)
            if entry.is_symlink() {
                continue;
            }

            let entry_path = canonical_dest.join(&entry_name);
            
            // Check if it escapes dest directory
            let safe = if let Some(parent) = entry_path.parent() {
                std::fs::create_dir_all(parent)?;
                if let Ok(canon_parent) = parent.canonicalize() {
                     canon_parent.starts_with(&canonical_dest)
                } else {
                    false
                }
            } else {
                false
            };
            
            if !safe {
                continue;
            }

            if entry.is_dir() {
                std::fs::create_dir_all(&entry_path)?;
            } else {
                if entry_path.exists() {
                    return Err(CompressionError::DestinationFileExistsError(
                        format!("Destination '{}' already exists", entry_path.display())
                    ));
                }
                
                let mut out_file = File::create(&entry_path)?;
                let mut buffer = vec![0; chunk_size];
                
                loop {
                    let bytes_read = entry.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    
                    if bytes_read as u64 > remaining_size {
                        return Err(CompressionError::ExtractFailed);
                    }
                    remaining_size -= bytes_read as u64;
                    
                    out_file.write_all(&buffer[..bytes_read])?;
                }
            }
        }
        
        Ok(dest_path.to_string_lossy().to_string())
    }
}
