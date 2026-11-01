use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedReport {
    pub source: String,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub label: Option<String>,
    pub url: Option<String>,
}

impl ResolvedReport {
    pub fn key(&self) -> String {
        format!("{}:{}", self.source, self.identifier)
    }
}
