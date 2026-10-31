use std::path::Path;
use super::strategy::{Strategy, CompressionError};

pub struct Pipeline {
    strategies: Vec<Box<dyn Strategy>>,
}

impl Pipeline {
    pub fn new(strategies: Vec<Box<dyn Strategy>>) -> Self {
        Self { strategies }
    }
}

impl Strategy for Pipeline {
    fn extension(&self) -> &'static str {
        ""
    }

    fn can_handle(&self, file_name: &str) -> bool {
        let combined_extension: String = self.strategies.iter().map(|s| s.extension()).collect();
        file_name.contains(&combined_extension)
    }

    fn compress(&self, path: &Path, target_name: &str) -> Result<String, CompressionError> {
        let mut current_target = target_name.to_string();
        let mut final_path = String::new();

        for strategy in &self.strategies {
            let compressed_path = strategy.compress(path, &current_target)?;
            current_target = Path::new(&compressed_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            final_path = compressed_path;
        }

        Ok(final_path)
    }

    fn decompress(&self, dest_path: &Path, compressed_file_path: &Path, max_size: u64) -> Result<String, CompressionError> {
        let mut to_decompress = compressed_file_path.to_path_buf();
        let strategies_rev: Vec<_> = self.strategies.iter().rev().collect();

        for (i, strategy) in strategies_rev.iter().enumerate() {
            let next_compressed_file = strategy.decompress(dest_path, &to_decompress, max_size)?;

            if i > 0 && to_decompress != compressed_file_path && to_decompress.is_file() {
                let _ = std::fs::remove_file(&to_decompress);
            }

            to_decompress = Path::new(&next_compressed_file).to_path_buf();
        }

        Ok(to_decompress.to_string_lossy().to_string())
    }
}
