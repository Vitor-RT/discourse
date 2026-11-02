use regex::Regex;
use std::path::{Path, PathBuf};

pub struct ThemeMock {
    pub name: String,
}

pub struct ZipExporter {
    pub theme: ThemeMock,
    temp_folder: PathBuf,
    export_name: String,
}

impl ZipExporter {
    pub fn new(theme: ThemeMock) -> Self {
        let temp_folder = std::env::temp_dir().join(format!("discourse_theme_{}", uuid::Uuid::new_v4().simple()));

        let re = Regex::new(r"[^0-9a-z.\-]").unwrap();
        let mut export_name = re.replace_all(&theme.name.to_lowercase(), "-").to_string();

        if !export_name.starts_with("discourse") {
            export_name = format!("discourse-{}", export_name);
        }

        ZipExporter {
            theme,
            temp_folder,
            export_name,
        }
    }

    pub fn export_name(&self) -> &str {
        &self.export_name
    }

    pub fn destination_folder(&self) -> PathBuf {
        self.temp_folder.join(&self.export_name)
    }

    pub fn is_safe_path(&self, requested_path: &Path) -> bool {
        let _dest = self.destination_folder();
        let mut depth = 0;
        for component in requested_path.components() {
            match component {
                std::path::Component::ParentDir => {
                    depth -= 1;
                    if depth < 0 {
                        return false;
                    }
                },
                std::path::Component::Normal(_) => {
                    depth += 1;
                },
                _ => continue,
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_name_generation() {
        let theme = ThemeMock { name: "My Awesome Theme!".to_string() };
        let exporter = ZipExporter::new(theme);
        assert_eq!(exporter.export_name(), "discourse-my-awesome-theme-");
    }

    #[test]
    fn test_export_name_generation_already_discourse() {
        let theme = ThemeMock { name: "discourse-cool-theme".to_string() };
        let exporter = ZipExporter::new(theme);
        assert_eq!(exporter.export_name(), "discourse-cool-theme");
    }

    #[test]
    fn test_is_safe_path() {
        let theme = ThemeMock { name: "test".to_string() };
        let exporter = ZipExporter::new(theme);

        assert!(exporter.is_safe_path(Path::new("some/normal/path.scss")));
        assert!(exporter.is_safe_path(Path::new("path/with/../resolved.scss")));
        assert!(!exporter.is_safe_path(Path::new("../escaped.scss")));
        assert!(!exporter.is_safe_path(Path::new("a/../../escaped.scss")));
    }
}
