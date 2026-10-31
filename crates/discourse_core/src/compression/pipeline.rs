use std::fs;
use std::path::{Path, PathBuf};

use crate::compression::strategy::{ExtractionError, Strategy};

pub struct Pipeline {
    strategies: Vec<Box<dyn Strategy>>,
    extension: String,
}

impl Pipeline {
    pub fn new(strategies: Vec<Box<dyn Strategy>>) -> Self {
        let extension = strategies
            .iter()
            .map(|s| s.extension())
            .collect::<Vec<_>>()
            .join("");
        Self { strategies, extension }
    }
}

impl Strategy for Pipeline {
    fn extension(&self) -> &str {
        &self.extension
    }

    fn compress(&self, path: &Path, target_name: &str) -> Result<String, ExtractionError> {
        let mut current_target = target_name.to_string();
        let mut compressed_path = String::new();

        for strategy in &self.strategies {
            compressed_path = strategy.compress(path, &current_target)?;
            current_target = Path::new(&compressed_path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
        }

        Ok(compressed_path)
    }

    fn decompress(
        &self,
        dest_path: &Path,
        compressed_file_path: &Path,
        max_size: usize,
    ) -> Result<PathBuf, ExtractionError> {
        let mut to_decompress = compressed_file_path.to_path_buf();
        let mut first = true;

        for strategy in self.strategies.iter().rev() {
            let next_compressed_file = strategy.decompress(dest_path, &to_decompress, max_size)?;

            if !first {
                // Remove intermediate file
                let _ = fs::remove_file(&to_decompress);
                let _ = fs::remove_dir_all(&to_decompress);
            }
            first = false;

            to_decompress = next_compressed_file;
        }

        Ok(to_decompress)
    }
}
