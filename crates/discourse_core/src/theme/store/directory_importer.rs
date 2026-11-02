use std::fs;
use std::path::{Path, PathBuf};
use crate::theme::store::base_importer::Importer;

pub struct DirectoryImporter {
    theme_dir: PathBuf,
    temp_folder: PathBuf,
}

impl DirectoryImporter {
    pub fn new<P: AsRef<Path>>(theme_dir: P) -> Self {
        let temp_folder = std::env::temp_dir().join(format!("discourse_theme_{}", uuid::Uuid::new_v4().simple()));
        DirectoryImporter {
            theme_dir: theme_dir.as_ref().to_path_buf(),
            temp_folder,
        }
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

impl Importer for DirectoryImporter {
    fn import(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.temp_folder)?;

        let excluded_dirs = ["node_modules", "src", "spec"];

        for entry in fs::read_dir(&self.theme_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if excluded_dirs.contains(&file_name_str.as_ref()) {
                continue;
            }

            let dest = self.temp_folder.join(file_name);
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                copy_dir_all(entry.path(), dest)?;
            } else {
                fs::copy(entry.path(), dest)?;
            }
        }

        Ok(())
    }

    fn temp_folder(&self) -> PathBuf {
        self.temp_folder.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_import_ignores_directories() {
        let theme_dir = TempDir::new().unwrap();
        let theme_path = theme_dir.path();

        fs::write(theme_path.join("file.txt"), "hello").unwrap();

        let node_modules = theme_path.join("node_modules");
        fs::create_dir(&node_modules).unwrap();
        fs::write(node_modules.join("bad.txt"), "bad").unwrap();

        let src = theme_path.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("bad.txt"), "bad").unwrap();

        let spec = theme_path.join("spec");
        fs::create_dir(&spec).unwrap();
        fs::write(spec.join("bad.txt"), "bad").unwrap();

        let mut importer = DirectoryImporter::new(theme_path);
        importer.import().unwrap();

        let temp_folder = importer.temp_folder();
        assert!(temp_folder.join("file.txt").exists());
        assert!(!temp_folder.join("node_modules").exists());
        assert!(!temp_folder.join("src").exists());
        assert!(!temp_folder.join("spec").exists());
    }
}
