use std::path::Path;
use super::strategy::{Strategy, CompressionError};
use super::zip::Zip;
use super::tar::Tar;
use super::gzip::Gzip;
use super::pipeline::Pipeline;

pub struct Engine {
    strategy: Box<dyn Strategy>,
}

impl Engine {
    pub fn default_strategies() -> Vec<Box<dyn Strategy>> {
        vec![
            Box::new(Zip),
            Box::new(Pipeline::new(vec![Box::new(Tar), Box::new(Gzip)])),
            Box::new(Gzip),
            Box::new(Tar),
        ]
    }

    pub fn engine_for(filename: &str) -> Result<Self, CompressionError> {
        let strategies = Self::default_strategies();
        for strategy in strategies {
            if strategy.can_handle(filename) {
                return Ok(Self { strategy });
            }
        }
        
        Err(CompressionError::UnsupportedFileExtension)
    }

    pub fn extension(&self) -> &'static str {
        self.strategy.extension()
    }

    pub fn compress(&self, path: &Path, target_name: &str) -> Result<String, CompressionError> {
        self.strategy.compress(path, target_name)
    }

    pub fn decompress(&self, dest_path: &Path, compressed_file_path: &Path, max_size: u64) -> Result<String, CompressionError> {
        self.strategy.decompress(dest_path, compressed_file_path, max_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_unsupported_file_extension() {
        let result = Engine::engine_for("a_file.crazyext");
        assert!(matches!(result, Err(CompressionError::UnsupportedFileExtension)));
    }

    #[test]
    fn test_compress_and_decompress_zip() {
        let dir = tempdir().unwrap();
        let folder_name = "test_zip";
        let folder_path = dir.path().join(folder_name);
        fs::create_dir(&folder_path).unwrap();
        
        let a_path = folder_path.join("a");
        fs::create_dir(&a_path).unwrap();
        
        let mut f1 = fs::File::create(folder_path.join("hello.txt")).unwrap();
        f1.write_all(b"hello world").unwrap();
        
        let mut f2 = fs::File::create(a_path.join("inner")).unwrap();
        f2.write_all(b"hello world inner").unwrap();
        
        let engine = Engine::engine_for("test_zip.zip").unwrap();
        
        // compress
        let compressed_path = engine.compress(dir.path(), folder_name).unwrap();
        assert!(Path::new(&compressed_path).exists());
        
        fs::remove_dir_all(&folder_path).unwrap();
        
        // decompress
        let extract_location = dir.path().join("extract_location");
        engine.decompress(&extract_location, Path::new(&compressed_path), 10_000_000).unwrap();
        
        let extracted_hello = extract_location.join("hello.txt");
        assert_eq!(fs::read_to_string(extracted_hello).unwrap(), "hello world");
        
        let extracted_inner = extract_location.join("a").join("inner");
        assert_eq!(fs::read_to_string(extracted_inner).unwrap(), "hello world inner");
    }

    #[test]
    fn test_compress_and_decompress_tar_gz() {
        let dir = tempdir().unwrap();
        let folder_name = "test_targz";
        let folder_path = dir.path().join(folder_name);
        fs::create_dir(&folder_path).unwrap();
        
        let a_path = folder_path.join("a");
        fs::create_dir(&a_path).unwrap();
        
        let mut f1 = fs::File::create(folder_path.join("hello.txt")).unwrap();
        f1.write_all(b"hello world").unwrap();
        
        let mut f2 = fs::File::create(a_path.join("inner")).unwrap();
        f2.write_all(b"hello world inner").unwrap();
        
        let engine = Engine::engine_for("test_targz.tar.gz").unwrap();
        
        let compressed_path = engine.compress(dir.path(), folder_name).unwrap();
        assert!(Path::new(&compressed_path).exists());
        
        fs::remove_dir_all(&folder_path).unwrap();
        
        let extract_location = dir.path().join("extract_location");
        engine.decompress(&extract_location, Path::new(&compressed_path), 10_000_000).unwrap();
        
        let extracted_hello = extract_location.join(folder_name).join("hello.txt");
        assert_eq!(fs::read_to_string(extracted_hello).unwrap(), "hello world");
        
        let extracted_inner = extract_location.join(folder_name).join("a").join("inner");
        assert_eq!(fs::read_to_string(extracted_inner).unwrap(), "hello world inner");
    }
}
