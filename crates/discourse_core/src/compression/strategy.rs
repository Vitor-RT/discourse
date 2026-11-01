use std::path::Path;

#[derive(Debug)]
pub enum CompressionError {
    UnsupportedFileExtension,
    ExtractFailed,
    DestinationFileExistsError(String),
    EntryTooLargeError(String),
    MissingEntryError(String),
    TooManyEntriesError(String),
    SuspiciousEntryError(String),
    IoError(std::io::Error),
    ZipError(zip::result::ZipError),
    Other(String),
}

impl std::fmt::Display for CompressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedFileExtension => write!(f, "Unsupported file extension"),
            Self::ExtractFailed => write!(f, "Extract failed"),
            Self::DestinationFileExistsError(msg) => write!(f, "Destination file exists: {}", msg),
            Self::EntryTooLargeError(msg) => write!(f, "Entry too large: {}", msg),
            Self::MissingEntryError(msg) => write!(f, "Missing entry: {}", msg),
            Self::TooManyEntriesError(msg) => write!(f, "Too many entries: {}", msg),
            Self::SuspiciousEntryError(msg) => write!(f, "Suspicious entry: {}", msg),
            Self::IoError(err) => write!(f, "IO error: {}", err),
            Self::ZipError(err) => write!(f, "Zip error: {}", err),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CompressionError {}

impl From<std::io::Error> for CompressionError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<zip::result::ZipError> for CompressionError {
    fn from(err: zip::result::ZipError) -> Self {
        Self::ZipError(err)
    }
}

pub trait Strategy: Send + Sync {
    fn extension(&self) -> &'static str;

    fn can_handle(&self, file_name: &str) -> bool {
        file_name.contains(self.extension())
    }

    fn compress(&self, path: &Path, target_name: &str) -> Result<String, CompressionError>;
    fn decompress(&self, dest_path: &Path, compressed_file_path: &Path, max_size: u64) -> Result<String, CompressionError>;
}
