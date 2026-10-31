use std::path::{Component, Path, PathBuf};

pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024 * 2; // 2 MiB

#[derive(Debug)]
pub enum ExtractionError {
    ExtractFailed,
    DestinationFileExistsError(String),
    UnsupportedFileExtension,
    Io(std::io::Error),
    Other(String),
}

impl From<std::io::Error> for ExtractionError {
    fn from(err: std::io::Error) -> Self {
        ExtractionError::Io(err)
    }
}

pub trait Strategy: Send {
    fn extension(&self) -> &str;

    fn compress(&self, path: &Path, target_name: &str) -> Result<String, ExtractionError>;

    fn decompress(
        &self,
        dest_path: &Path,
        compressed_file_path: &Path,
        max_size: usize,
    ) -> Result<PathBuf, ExtractionError>;
}

pub fn sanitize_path(path: &Path) -> Result<PathBuf, ExtractionError> {
    path.canonicalize().map_err(ExtractionError::Io)
}

pub fn calculate_available_size(max_size: usize) -> usize {
    // max_size in MB to Bytes
    (1024.0 * 1024.0 * (max_size as f64 / 1.049)) as usize
}

// Normalize a path (evaluate .. and . similar to Ruby's File.expand_path but without resolving symlinks on disk if they don't exist yet)
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            _ => normalized.push(component),
        }
    }
    normalized
}

pub fn is_safe_path_for_extraction(path: &Path, dest_directory: &Path) -> bool {
    let Ok(canonical_dest) = dest_directory.canonicalize() else {
        return false;
    };

    let normalized_path = normalize_path(path);

    // Check if the normalized path starts with the canonical destination path
    normalized_path.starts_with(&canonical_dest)
}
