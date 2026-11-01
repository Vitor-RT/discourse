use std::fs::File;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use super::strategy::{Strategy, CompressionError};

pub struct Tar;

fn is_safe_path(path: &Path, dest_directory: &Path) -> bool {
    if let Ok(resolved) = path.canonicalize() {
        if let Ok(dest_resolved) = dest_directory.canonicalize() {
            return resolved.starts_with(dest_resolved);
        }
    }
    false
}

impl Strategy for Tar {
    fn extension(&self) -> &'static str {
        ".tar"
    }

    fn compress(&self, path: &Path, target_name: &str) -> Result<String, CompressionError> {
        let source_path = path.join(target_name);

        let tar_filename = format!("{}.tar", target_name);
        // Replace invalid chars as per FileHelper.sanitize_filename in ruby
        let tar_filename = tar_filename.replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_', "_");

        let target_path = path.join(&tar_filename);

        let file = File::create(&target_path)?;
        let mut builder = Builder::new(file);

        if source_path.is_dir() {
            builder.append_dir_all(target_name, &source_path)?;
        } else {
            let mut source_file = File::open(&source_path)?;
            builder.append_file(target_name, &mut source_file)?;
        }

        builder.finish()?;

        Ok(target_path.to_string_lossy().to_string())
    }

    fn decompress(&self, dest_path: &Path, compressed_file_path: &Path, max_size: u64) -> Result<String, CompressionError> {
        let file = File::open(compressed_file_path)?;
        let mut archive = Archive::new(file);

        std::fs::create_dir_all(dest_path)?;
        let canonical_dest = dest_path.canonicalize()?;

        for entry in archive.entries()? {
            let mut entry = entry?;
            let entry_path = entry.path()?.into_owned();

            let target_path = canonical_dest.join(&entry_path);

            // For security, only extract inside dest_path (avoid path traversal)
            let mut safe_to_extract = false;
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
                if let Ok(canon_parent) = parent.canonicalize() {
                     if canon_parent.starts_with(&canonical_dest) {
                         safe_to_extract = true;
                     }
                }
            }

            if !safe_to_extract {
                continue;
            }

            let entry_type = entry.header().entry_type();
            if !entry_type.is_file() && !entry_type.is_dir() {
                continue;
            }

            entry.unpack(&target_path)?;
        }

        Ok(dest_path.to_string_lossy().to_string())
    }
}
