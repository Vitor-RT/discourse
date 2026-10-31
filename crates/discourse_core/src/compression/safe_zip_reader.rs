use std::io::{Read, Seek, Write};
use zip::ZipArchive;

#[derive(Debug)]
pub enum SafeZipError {
    EntryTooLargeError,
    MissingEntryError,
    TooManyEntriesError,
    SuspiciousEntryError,
    Io(std::io::Error),
    ZipError(zip::result::ZipError),
}

impl From<std::io::Error> for SafeZipError {
    fn from(err: std::io::Error) -> Self {
        SafeZipError::Io(err)
    }
}

impl From<zip::result::ZipError> for SafeZipError {
    fn from(err: zip::result::ZipError) -> Self {
        SafeZipError::ZipError(err)
    }
}

pub struct SafeZipReader<R: Read + Seek> {
    pub archive: ZipArchive<R>,
    pub max_entries: Option<usize>,
    pub max_total_bytes: Option<usize>,
    pub max_compression_ratio: Option<f64>,
    pub read_chunk_bytes: usize,
}

const DEFAULT_READ_CHUNK_BYTES: usize = 16 * 1024;

impl<R: Read + Seek> SafeZipReader<R> {
    pub fn new(
        archive: ZipArchive<R>,
        max_entries: Option<usize>,
        max_total_bytes: Option<usize>,
        max_compression_ratio: Option<f64>,
    ) -> Result<Self, SafeZipError> {
        let reader = SafeZipReader {
            archive,
            max_entries,
            max_total_bytes,
            max_compression_ratio,
            read_chunk_bytes: DEFAULT_READ_CHUNK_BYTES,
        };
        reader.validate()?;
        Ok(reader)
    }

    pub fn validate(&self) -> Result<(), SafeZipError> {
        if let Some(max_entries) = self.max_entries {
            if self.archive.len() > max_entries {
                return Err(SafeZipError::TooManyEntriesError);
            }
        }
        Ok(())
    }

    fn entry_limit(max_total_bytes: Option<usize>, max_bytes: usize) -> Result<usize, SafeZipError> {
        let limit = match max_total_bytes {
            Some(remaining) => max_bytes.min(remaining),
            None => max_bytes,
        };
        if limit == 0 {
            return Err(SafeZipError::EntryTooLargeError);
        }
        Ok(limit)
    }

    fn validate_entry_metadata(
        size: usize,
        compressed_size: usize,
        limit: usize,
        max_compression_ratio: Option<f64>,
    ) -> Result<(), SafeZipError> {
        if size > 0 && size > limit {
            return Err(SafeZipError::EntryTooLargeError);
        }

        if let Some(max_ratio) = max_compression_ratio {
            if compressed_size > 0 && size > 0 {
                if (size as f64 / compressed_size as f64) > max_ratio {
                    return Err(SafeZipError::SuspiciousEntryError);
                }
            }
        }

        Ok(())
    }

    pub fn read_entry(
        &mut self,
        name: &str,
        max_bytes: usize,
        required: bool,
    ) -> Result<Option<Vec<u8>>, SafeZipError> {
        let mut entry = match self.archive.by_name(name) {
            Ok(e) => e,
            Err(zip::result::ZipError::FileNotFound) => {
                if required {
                    return Err(SafeZipError::MissingEntryError);
                }
                return Ok(None);
            }
            Err(e) => return Err(SafeZipError::ZipError(e)),
        };

        if entry.is_dir() {
            if required {
                return Err(SafeZipError::MissingEntryError);
            }
            return Ok(None);
        }

        let size = entry.size() as usize;
        let compressed_size = entry.compressed_size() as usize;

        let limit = Self::entry_limit(self.max_total_bytes, max_bytes)?;
        Self::validate_entry_metadata(size, compressed_size, limit, self.max_compression_ratio)?;

        let mut data = Vec::new();
        let mut bytes_read = 0;
        let mut buffer = vec![0; self.read_chunk_bytes];

        loop {
            let n = entry.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            bytes_read += n;
            if bytes_read > limit {
                return Err(SafeZipError::EntryTooLargeError);
            }
            data.extend_from_slice(&buffer[..n]);
        }

        if let Some(ref mut remaining) = self.max_total_bytes {
            *remaining = remaining.saturating_sub(bytes_read);
        }

        Ok(Some(data))
    }

    pub fn stream_entry_to_file<W: Write>(
        &mut self,
        name: &str,
        file: &mut W,
        max_bytes: usize,
        required: bool,
    ) -> Result<(), SafeZipError> {
        let mut entry = match self.archive.by_name(name) {
            Ok(e) => e,
            Err(zip::result::ZipError::FileNotFound) => {
                if required {
                    return Err(SafeZipError::MissingEntryError);
                }
                return Ok(());
            }
            Err(e) => return Err(SafeZipError::ZipError(e)),
        };

        if entry.is_dir() {
            if required {
                return Err(SafeZipError::MissingEntryError);
            }
            return Ok(());
        }

        let size = entry.size() as usize;
        let compressed_size = entry.compressed_size() as usize;

        let limit = Self::entry_limit(self.max_total_bytes, max_bytes)?;
        Self::validate_entry_metadata(size, compressed_size, limit, self.max_compression_ratio)?;

        let mut bytes_read = 0;
        let mut buffer = vec![0; self.read_chunk_bytes];

        loop {
            let n = entry.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            bytes_read += n;
            if bytes_read > limit {
                return Err(SafeZipError::EntryTooLargeError);
            }
            file.write_all(&buffer[..n]).map_err(SafeZipError::Io)?;
        }

        if let Some(ref mut remaining) = self.max_total_bytes {
            *remaining = remaining.saturating_sub(bytes_read);
        }

        Ok(())
    }
}
