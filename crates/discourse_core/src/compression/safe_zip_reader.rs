use std::fs::File;
use zip::ZipArchive;
use super::strategy::CompressionError;

pub struct SafeZipReader {
    pub archive: ZipArchive<File>,
    pub max_entries: Option<usize>,
    pub remaining_total_bytes: Option<u64>,
    pub max_compression_ratio: Option<f64>,
}

impl SafeZipReader {
    pub fn new(
        file: File,
        max_entries: Option<usize>,
        max_total_bytes: Option<u64>,
        max_compression_ratio: Option<f64>,
    ) -> Result<Self, CompressionError> {
        let mut archive = ZipArchive::new(file)?;

        if let Some(max_entries) = max_entries {
            if archive.len() > max_entries {
                return Err(CompressionError::TooManyEntriesError(
                    "Zip archive has too many entries".to_string(),
                ));
            }
        }

        if let Some(ratio) = max_compression_ratio {
            for i in 0..archive.len() {
                let entry = archive.by_index(i)?;
                let compressed = entry.compressed_size();
                let uncompressed = entry.size();

                if compressed > 0 && (uncompressed as f64 / compressed as f64) > ratio {
                    return Err(CompressionError::SuspiciousEntryError(format!(
                        "Zip entry {} has suspicious compression ratio",
                        entry.name()
                    )));
                }
            }
        }

        Ok(Self {
            archive,
            max_entries,
            remaining_total_bytes: max_total_bytes,
            max_compression_ratio,
        })
    }

    pub fn len(&self) -> usize {
        self.archive.len()
    }

    pub fn by_index(&mut self, i: usize) -> Result<zip::read::ZipFile<'_, File>, CompressionError> {
        let entry = self.archive.by_index(i)?;

        if let Some(limit) = self.remaining_total_bytes {
            if entry.size() > limit {
                return Err(CompressionError::EntryTooLargeError(format!(
                    "Zip entry {} is too large",
                    entry.name()
                )));
            }
        }

        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;

    #[test]
    fn test_valid_zip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.zip");
        let file = File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("test.txt", options).unwrap();
        zip.write_all(b"hello world").unwrap();
        zip.finish().unwrap();

        let file = File::open(&path).unwrap();
        let reader = SafeZipReader::new(file, Some(10), Some(100), None).unwrap();
        assert_eq!(reader.len(), 1);
    }

    #[test]
    fn test_too_many_entries() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.zip");
        let file = File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("test.txt", options).unwrap();
        zip.write_all(b"hello world").unwrap();
        zip.finish().unwrap();

        let file = File::open(&path).unwrap();
        let result = SafeZipReader::new(file, Some(0), Some(100), None);
        assert!(matches!(
            result,
            Err(CompressionError::TooManyEntriesError(_))
        ));
    }
}
