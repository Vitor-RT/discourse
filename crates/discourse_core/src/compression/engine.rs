use std::path::{Path, PathBuf};

use crate::compression::strategy::{ExtractionError, Strategy};
use crate::compression::gzip::Gzip;
use crate::compression::pipeline::Pipeline;
use crate::compression::tar::Tar;
use crate::compression::zip::Zip;

pub struct CompressionEngine {
    strategy: Box<dyn Strategy>,
}

impl CompressionEngine {
    pub fn default_strategies() -> Vec<Box<dyn Strategy>> {
        vec![
            Box::new(Zip),
            Box::new(Pipeline::new(vec![Box::new(Tar), Box::new(Gzip)])),
            Box::new(Gzip),
            Box::new(Tar),
        ]
    }

    pub fn engine_for(filename: &str) -> Result<Self, ExtractionError> {
        let strategies = Self::default_strategies();
        for strategy in strategies {
            if filename.contains(strategy.extension()) {
                return Ok(Self { strategy });
            }
        }
        Err(ExtractionError::UnsupportedFileExtension)
    }

    pub fn extension(&self) -> &str {
        self.strategy.extension()
    }

    pub fn compress(&self, path: &Path, target_name: &str) -> Result<String, ExtractionError> {
        self.strategy.compress(path, target_name)
    }

    pub fn decompress(
        &self,
        dest_path: &Path,
        compressed_file_path: &Path,
        max_size: usize,
    ) -> Result<PathBuf, ExtractionError> {
        self.strategy.decompress(dest_path, compressed_file_path, max_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_for() {
        assert!(CompressionEngine::engine_for("file.zip").is_ok());
        assert!(CompressionEngine::engine_for("file.tar.gz").is_ok());
        assert!(CompressionEngine::engine_for("file.gz").is_ok());
        assert!(CompressionEngine::engine_for("file.tar").is_ok());
        assert!(CompressionEngine::engine_for("file.txt").is_err());
    }
}
