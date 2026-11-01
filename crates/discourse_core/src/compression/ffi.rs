use magnus::{function, method, prelude::*, Error, Ruby};

use crate::compression::engine::CompressionEngine;
use crate::compression::strategy::ExtractionError;
use std::path::Path;

use crate::compression::safe_zip_reader::{SafeZipReader, SafeZipError};
use std::fs::File;
use zip::ZipArchive;

fn map_error(ruby: &Ruby, err: ExtractionError) -> Error {
    Error::new(ruby.exception_standard_error(), format!("{:?}", err))
}

fn map_zip_error(ruby: &Ruby, err: SafeZipError) -> Error {
    Error::new(ruby.exception_standard_error(), format!("{:?}", err))
}

#[magnus::wrap(class = "Compression::Engine")]
struct EngineWrapper {
    engine: CompressionEngine,
}

impl EngineWrapper {
    fn new(ruby: &Ruby, filename: String) -> Result<Self, Error> {
        let engine = CompressionEngine::engine_for(&filename).map_err(|e| map_error(ruby, e))?;
        Ok(Self { engine })
    }

    fn extension(&self) -> String {
        self.engine.extension().to_string()
    }

    fn compress(&self, path: String, target_name: String) -> Result<String, Error> {
        let ruby = Ruby::get().unwrap();
        self.engine
            .compress(Path::new(&path), &target_name)
            .map_err(|e| map_error(&ruby, e))
    }

    fn decompress(
        &self,
        dest_path: String,
        compressed_file_path: String,
        max_size: usize,
    ) -> Result<String, Error> {
        let ruby = Ruby::get().unwrap();
        let path = self
            .engine
            .decompress(Path::new(&dest_path), Path::new(&compressed_file_path), max_size)
            .map_err(|e| map_error(&ruby, e))?;
        Ok(path.to_string_lossy().to_string())
    }
}

#[magnus::wrap(class = "Compression::SafeZipReader")]
struct SafeZipReaderWrapper {
    max_entries: Option<usize>,
    max_total_bytes: Option<usize>,
    max_compression_ratio: Option<f64>,
}

impl SafeZipReaderWrapper {
    fn new(
        max_entries: Option<usize>,
        max_total_bytes: Option<usize>,
        max_compression_ratio: Option<f64>,
    ) -> Self {
        Self {
            max_entries,
            max_total_bytes,
            max_compression_ratio,
        }
    }

    fn validate(&self, path: String) -> Result<(), Error> {
        let ruby = Ruby::get().unwrap();
        let file = File::open(&path).map_err(|e| Error::new(ruby.exception_standard_error(), e.to_string()))?;
        let archive = ZipArchive::new(file).map_err(|e| Error::new(ruby.exception_standard_error(), e.to_string()))?;

        let reader = SafeZipReader::new(
            archive,
            self.max_entries,
            self.max_total_bytes,
            self.max_compression_ratio
        ).map_err(|e| map_zip_error(&ruby, e))?;

        reader.validate().map_err(|e| map_zip_error(&ruby, e))
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Compression")?;

    let engine_class = module.define_class("Engine", ruby.class_object())?;
    engine_class.define_singleton_method("engine_for", function!(EngineWrapper::new, 1))?;
    engine_class.define_method("extension", method!(EngineWrapper::extension, 0))?;
    engine_class.define_method("compress", method!(EngineWrapper::compress, 2))?;
    engine_class.define_method("decompress", method!(EngineWrapper::decompress, 3))?;

    let safe_zip_class = module.define_class("SafeZipReader", ruby.class_object())?;
    safe_zip_class.define_singleton_method("new", function!(SafeZipReaderWrapper::new, 3))?;
    safe_zip_class.define_method("validate", method!(SafeZipReaderWrapper::validate, 1))?;

    Ok(())
}
