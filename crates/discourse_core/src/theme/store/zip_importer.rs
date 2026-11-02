use crate::theme::store::base_importer::Importer;
use std::fs;
use std::path::PathBuf;

pub struct ZipImporter {
    pub filename: String,
    pub original_filename: String,
    temp_folder: PathBuf,
}

impl ZipImporter {
    pub fn new(filename: String, original_filename: String) -> Self {
        let temp_folder =
            std::env::temp_dir().join(format!("discourse_theme_{}", uuid::Uuid::new_v4().simple()));
        ZipImporter {
            filename,
            original_filename,
            temp_folder,
        }
    }

    pub fn strip_root_directory(&self) -> std::io::Result<()> {
        let entries: Vec<_> = fs::read_dir(&self.temp_folder)?
            .filter_map(|e| e.ok())
            .collect();

        if entries.len() == 1 {
            if let Ok(file_type) = entries[0].file_type() {
                if file_type.is_dir() {
                    let root_dir = entries[0].path();
                    let root_entries: Vec<_> =
                        fs::read_dir(&root_dir)?.filter_map(|e| e.ok()).collect();

                    for entry in root_entries {
                        let new_path = self.temp_folder.join(entry.file_name());
                        fs::rename(entry.path(), new_path)?;
                    }
                    fs::remove_dir(root_dir)?;
                }
            }
        }
        Ok(())
    }
}

impl Importer for ZipImporter {
    fn import(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.temp_folder)?;
        self.strip_root_directory()?;
        Ok(())
    }

    fn temp_folder(&self) -> PathBuf {
        self.temp_folder.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_root_directory() {
        let importer = ZipImporter::new("file.zip".into(), "orig.zip".into());
        let temp = importer.temp_folder();
        fs::create_dir_all(&temp).unwrap();

        let root = temp.join("discourse-theme");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("file.txt"), "hello").unwrap();

        importer.strip_root_directory().unwrap();

        assert!(temp.join("file.txt").exists());
        assert!(!temp.join("discourse-theme").exists());
    }
}
