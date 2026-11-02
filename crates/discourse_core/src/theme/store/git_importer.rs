use std::collections::HashMap;
use std::path::PathBuf;

use crate::theme::store::base_importer::Importer;

pub struct GitImporter {
    pub url: String,
    pub private_key: Option<String>,
    pub branch: Option<String>,
    temp_folder: PathBuf,
}

impl GitImporter {
    pub fn new(url: String, private_key: Option<String>, branch: Option<String>) -> Self {
        let temp_folder =
            std::env::temp_dir().join(format!("discourse_theme_{}", uuid::Uuid::new_v4().simple()));
        GitImporter {
            url,
            private_key,
            branch,
            temp_folder,
        }
    }

    pub fn clone_args(&self, url: &str, config: &HashMap<String, String>) -> Vec<String> {
        let mut args = vec!["git".to_string()];

        for (key, value) in config {
            args.push("-c".to_string());
            args.push(format!("{}={}", key, value));
        }

        args.push("clone".to_string());

        if let Some(ref branch) = self.branch {
            if !branch.is_empty() {
                args.push("--single-branch".to_string());
                args.push("-b".to_string());
                args.push(branch.clone());
            }
        }

        args.push(url.to_string());
        args.push(self.temp_folder.to_string_lossy().to_string());

        args
    }
}

impl Importer for GitImporter {
    fn import(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Translation focuses on pure logic extraction like clone_args.
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
    fn test_clone_args_basic() {
        let importer = GitImporter::new("https://example.com/repo.git".into(), None, None);
        let args = importer.clone_args("https://example.com/repo.git", &HashMap::new());
        let expected: Vec<String> = vec![
            "git".into(),
            "clone".into(),
            "https://example.com/repo.git".into(),
            importer.temp_folder().to_string_lossy().to_string(),
        ];
        assert_eq!(args, expected);
    }

    #[test]
    fn test_clone_args_with_branch() {
        let importer = GitImporter::new(
            "https://example.com/repo.git".into(),
            None,
            Some("main".into()),
        );
        let args = importer.clone_args("https://example.com/repo.git", &HashMap::new());
        let expected: Vec<String> = vec![
            "git".into(),
            "clone".into(),
            "--single-branch".into(),
            "-b".into(),
            "main".into(),
            "https://example.com/repo.git".into(),
            importer.temp_folder().to_string_lossy().to_string(),
        ];
        assert_eq!(args, expected);
    }
}
